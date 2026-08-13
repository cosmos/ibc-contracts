// SPDX-License-Identifier: Apache-2.0

//! Procedural macros for IBC applications on Solana
//!
//! This crate provides:
//! - `#[ibc_app]` - Validates IBC app callback implementations
//! - `discriminator!` - Computes Anchor discriminators at compile time

use proc_macro::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use syn::{
    parse_macro_input, FnArg, GenericArgument, ItemFn, ItemMod, LitStr, PathArguments, ReturnType,
    Type, TypePath, TypeTuple,
};

// ============================================================================
// Constants & Types
// ============================================================================

/// Configuration for each IBC callback
#[derive(Debug, Clone)]
struct CallbackConfig {
    /// The function name
    name: &'static str,
    /// Expected message type name
    msg_type: &'static str,
    /// Expected return type
    return_type: ReturnTypeExpectation,
    /// Discriminator name (for compile-time checks)
    discriminator_name: &'static str,
}

#[derive(Debug, Clone, Copy)]
enum ReturnTypeExpectation {
    ResultUnit,
    ResultVecU8,
}

impl CallbackConfig {
    const fn new(
        name: &'static str,
        msg_type: &'static str,
        return_type: ReturnTypeExpectation,
        discriminator_name: &'static str,
    ) -> Self {
        Self {
            name,
            msg_type,
            return_type,
            discriminator_name,
        }
    }
}

/// IBC callback configurations
const IBC_CALLBACKS: &[CallbackConfig] = &[
    CallbackConfig::new(
        "on_recv_packet",
        "OnRecvPacketMsg",
        ReturnTypeExpectation::ResultVecU8,
        "OnRecvPacket",
    ),
    CallbackConfig::new(
        "on_acknowledgement_packet",
        "OnAcknowledgementPacketMsg",
        ReturnTypeExpectation::ResultUnit,
        "OnAcknowledgementPacket",
    ),
    CallbackConfig::new(
        "on_timeout_packet",
        "OnTimeoutPacketMsg",
        ReturnTypeExpectation::ResultUnit,
        "OnTimeoutPacket",
    ),
];

// ============================================================================
// Main Macro Entry Point
// ============================================================================

/// Attribute macro for IBC applications
///
/// This macro wraps Anchor's `#[program]` macro and adds compile-time validation
/// to ensure all required IBC callback functions are implemented with correct names.
///
/// # Required Callbacks
///
/// Your IBC app MUST implement these three functions with the exact
/// return types shown below:
///
/// 1. `on_recv_packet` → `Result<Vec<u8>>` — Handle incoming packets and return acknowledgement bytes
/// 2. `on_acknowledgement_packet` → `Result<()>` — Handle acknowledgements for sent packets (NOT `on_ack_packet`)
/// 3. `on_timeout_packet` → `Result<()>` — Handle timeouts for sent packets
///
/// # Example
///
/// ```ignore
/// use solana_ibc_macros::ibc_app;
///
/// declare_id!("...");
///
/// #[ibc_app]
/// pub mod my_ibc_app {
///     use super::*;
///
///     pub fn on_recv_packet<'info>(
///         ctx: Context<'_, '_, '_, 'info, OnRecvPacket<'info>>,
///         msg: OnRecvPacketMsg,
///     ) -> Result<Vec<u8>> {
///         // Handle received packet
///         Ok(vec![])
///     }
///
///     pub fn on_acknowledgement_packet(
///         ctx: Context<OnAckPacket>,
///         msg: OnAcknowledgementPacketMsg,
///     ) -> Result<()> {
///         // Handle acknowledgement
///         Ok(())
///     }
///
///     pub fn on_timeout_packet(
///         ctx: Context<OnTimeoutPacket>,
///         msg: OnTimeoutPacketMsg,
///     ) -> Result<()> {
///         // Handle timeout
///         Ok(())
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn ibc_app(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let module = parse_macro_input!(item as ItemMod);

    // Validate that all required callbacks are present with correct signatures
    let validator = CallbackValidator::new(&module);
    if let Err(e) = validator.validate() {
        return e.to_compile_error().into();
    }

    // Generate output with compile-time assertions
    generate_output(module)
}

// ============================================================================
// Validation Logic
// ============================================================================

struct CallbackValidator<'a> {
    module: &'a ItemMod,
    functions: HashMap<String, &'a ItemFn>,
}

impl<'a> CallbackValidator<'a> {
    fn new(module: &'a ItemMod) -> Self {
        let functions = Self::collect_functions(module);
        Self { module, functions }
    }

    fn validate(&self) -> syn::Result<()> {
        // Ensure module has content
        self.validate_module_content()?;

        // Check for missing callbacks
        self.validate_missing_callbacks()?;

        // Validate signatures of present callbacks
        self.validate_callback_signatures()?;

        Ok(())
    }

    fn collect_functions(module: &'a ItemMod) -> HashMap<String, &'a ItemFn> {
        let mut functions = HashMap::new();

        if let Some((_, items)) = &module.content {
            for item in items {
                if let syn::Item::Fn(item_fn) = item {
                    functions.insert(item_fn.sig.ident.to_string(), item_fn);
                }
            }
        }

        functions
    }

    fn validate_module_content(&self) -> syn::Result<()> {
        if self.module.content.is_none() {
            return Err(syn::Error::new_spanned(
                self.module,
                "IBC app module must have a body with callback implementations",
            ));
        }
        Ok(())
    }

    fn validate_missing_callbacks(&self) -> syn::Result<()> {
        let missing_callbacks: Vec<_> = IBC_CALLBACKS
            .iter()
            .filter(|cb| !self.functions.contains_key(cb.name))
            .collect();

        if missing_callbacks.is_empty() {
            return Ok(());
        }

        let error_msg = self.build_missing_callbacks_error(&missing_callbacks);
        Err(syn::Error::new_spanned(self.module, error_msg))
    }

    fn build_missing_callbacks_error(&self, missing: &[&CallbackConfig]) -> String {
        let missing_names: Vec<_> = missing.iter().map(|cb| cb.name).collect();
        let mut error_msg = format!(
            "IBC app is missing required callback function(s): {}",
            missing_names.join(", ")
        );

        // Check for common naming mistakes
        if let Some(suggestion) = self.check_common_mistakes(&missing_names) {
            error_msg.push_str(&suggestion);
        }

        error_msg
    }

    fn check_common_mistakes(&self, missing: &[&str]) -> Option<String> {
        // Check for shortened acknowledgement callback name
        if missing.contains(&"on_acknowledgement_packet")
            && self.functions.contains_key("on_ack_packet")
        {
            return Some(
                "\n\nFound 'on_ack_packet' but expected 'on_acknowledgement_packet'.\n\
                 The router expects the full name 'on_acknowledgement_packet', not 'on_ack_packet'.".into()
            );
        }

        // Could add more common mistake patterns here
        None
    }

    fn validate_callback_signatures(&self) -> syn::Result<()> {
        for callback in IBC_CALLBACKS {
            if let Some(item_fn) = self.functions.get(callback.name) {
                SignatureValidator::new(callback, item_fn).validate()?;
            }
        }
        Ok(())
    }
}

// ============================================================================
// Signature Validation
// ============================================================================

struct SignatureValidator<'a> {
    config: &'a CallbackConfig,
    item_fn: &'a ItemFn,
}

impl<'a> SignatureValidator<'a> {
    const fn new(config: &'a CallbackConfig, item_fn: &'a ItemFn) -> Self {
        Self { config, item_fn }
    }

    fn validate(&self) -> syn::Result<()> {
        self.validate_parameter_count()?;
        self.validate_return_type()?;
        self.validate_message_parameter()?;
        Ok(())
    }

    fn validate_parameter_count(&self) -> syn::Result<()> {
        let sig = &self.item_fn.sig;
        if sig.inputs.len() != 2 {
            return Err(syn::Error::new_spanned(
                &sig.inputs,
                format!(
                    "Callback '{}' must have exactly 2 parameters (ctx: Context<...>, msg: {}), found {}",
                    self.config.name,
                    self.config.msg_type,
                    sig.inputs.len()
                ),
            ));
        }
        Ok(())
    }

    fn validate_return_type(&self) -> syn::Result<()> {
        let sig = &self.item_fn.sig;
        let return_type = &sig.output;

        let is_valid = match self.config.return_type {
            ReturnTypeExpectation::ResultUnit => is_result_unit(return_type),
            ReturnTypeExpectation::ResultVecU8 => is_result_vec_u8(return_type),
        };

        if !is_valid {
            let expected = match self.config.return_type {
                ReturnTypeExpectation::ResultUnit => "Result<()>",
                ReturnTypeExpectation::ResultVecU8 => "Result<Vec<u8>>",
            };
            return Err(syn::Error::new_spanned(
                return_type,
                format!(
                    "Callback '{}' must return '{}', found '{}'",
                    self.config.name,
                    expected,
                    quote::quote!(#return_type)
                ),
            ));
        }

        Ok(())
    }

    fn validate_message_parameter(&self) -> syn::Result<()> {
        let sig = &self.item_fn.sig;

        if let Some(FnArg::Typed(pat_type)) = sig.inputs.iter().nth(1) {
            if !type_ends_with(&pat_type.ty, self.config.msg_type) {
                let actual_ty = &pat_type.ty;
                return Err(syn::Error::new_spanned(
                    actual_ty,
                    format!(
                        "Callback '{}' second parameter must be of type '{}', found '{}'",
                        self.config.name,
                        self.config.msg_type,
                        quote::quote!(#actual_ty)
                    ),
                ));
            }
        }

        Ok(())
    }
}

// ============================================================================
// Type Checking Utilities
// ============================================================================

/// Extract the first type argument from a `Result<T>` return type.
fn extract_result_inner_type(return_type: &ReturnType) -> Option<&Type> {
    let ReturnType::Type(_, ty) = return_type else {
        return None;
    };
    let Type::Path(TypePath { path, .. }) = &**ty else {
        return None;
    };
    let segment = path.segments.last()?;
    if segment.ident != "Result" {
        return None;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        // Anchor's `Result<T>` is a type alias, so bare `Result` (no angle
        // brackets) can appear when the user writes e.g. `anchor_lang::Result`.
        // We can't inspect the inner type without brackets, so reject it.
        return None;
    };
    match args.args.first() {
        Some(GenericArgument::Type(ty)) => Some(ty),
        _ => None,
    }
}

/// Check if return type is `Result<Vec<u8>>`.
fn is_result_vec_u8(return_type: &ReturnType) -> bool {
    let Some(inner) = extract_result_inner_type(return_type) else {
        return false;
    };
    let Type::Path(TypePath { path, .. }) = inner else {
        return false;
    };
    let Some(segment) = path.segments.last() else {
        return false;
    };
    if segment.ident != "Vec" {
        return false;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    matches!(
        args.args.first(),
        Some(GenericArgument::Type(Type::Path(TypePath { path, .. })))
            if path.segments.last().is_some_and(|s| s.ident == "u8")
    )
}

/// Check if return type is `Result<()>`.
fn is_result_unit(return_type: &ReturnType) -> bool {
    let Some(inner) = extract_result_inner_type(return_type) else {
        return false;
    };
    matches!(inner, Type::Tuple(TypeTuple { elems, .. }) if elems.is_empty())
}

/// Check if a type path ends with the expected identifier
fn type_ends_with(ty: &Type, expected: &str) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == expected;
        }
    }
    false
}

// ============================================================================
// Code Generation
// ============================================================================

fn generate_output(module: ItemMod) -> TokenStream {
    let discriminator_checks = generate_discriminator_checks();

    let output = quote! {
        #[::anchor_lang::program]
        #module

        // Compile-time check that instruction discriminators exist with correct names
        const _: () = {
            use ::anchor_lang::Discriminator;
            #discriminator_checks
        };
    };

    TokenStream::from(output)
}

fn generate_discriminator_checks() -> proc_macro2::TokenStream {
    let checks = IBC_CALLBACKS.iter().map(|callback| {
        let discriminator_path = format!("crate::instruction::{}", callback.discriminator_name);
        let discriminator_ident: proc_macro2::TokenStream = discriminator_path.parse().unwrap();

        quote! {
            // Verify #callback.name discriminator exists
            let _ = #discriminator_ident::DISCRIMINATOR;
        }
    });

    quote! {
        #(#checks)*
    }
}

// ============================================================================
// Discriminator Macro
// ============================================================================

/// Computes an Anchor instruction discriminator at compile time.
///
/// The discriminator is the first 8 bytes of `sha256("global:<instruction_name>")`.
///
/// # Example
///
/// ```ignore
/// use solana_ibc_macros::discriminator;
///
/// const ON_RECV_PACKET_DISCRIMINATOR: [u8; 8] = discriminator!("on_recv_packet");
/// ```
#[proc_macro]
pub fn discriminator(input: TokenStream) -> TokenStream {
    let instruction_name = parse_macro_input!(input as LitStr).value();

    let preimage = format!("global:{instruction_name}");
    let hash = Sha256::digest(preimage.as_bytes());
    let bytes: [u8; 8] = hash[..8].try_into().unwrap();

    let b0 = bytes[0];
    let b1 = bytes[1];
    let b2 = bytes[2];
    let b3 = bytes[3];
    let b4 = bytes[4];
    let b5 = bytes[5];
    let b6 = bytes[6];
    let b7 = bytes[7];

    let output = quote! {
        [#b0, #b1, #b2, #b3, #b4, #b5, #b6, #b7]
    };

    output.into()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    fn return_type(ty: Type) -> ReturnType {
        parse_quote!(-> #ty)
    }

    // --- is_result_vec_u8 ---

    #[test]
    fn accepts_result_vec_u8() {
        let rt = return_type(parse_quote!(Result<Vec<u8>>));
        assert!(is_result_vec_u8(&rt));
    }

    #[test]
    fn accepts_qualified_result_vec_u8() {
        let rt = return_type(parse_quote!(anchor_lang::Result<Vec<u8>>));
        assert!(is_result_vec_u8(&rt));
    }

    #[test]
    fn rejects_result_unit_for_vec_u8() {
        let rt = return_type(parse_quote!(Result<()>));
        assert!(!is_result_vec_u8(&rt));
    }

    #[test]
    fn rejects_result_string_for_vec_u8() {
        let rt = return_type(parse_quote!(Result<String>));
        assert!(!is_result_vec_u8(&rt));
    }

    #[test]
    fn rejects_result_vec_u16_for_vec_u8() {
        let rt = return_type(parse_quote!(Result<Vec<u16>>));
        assert!(!is_result_vec_u8(&rt));
    }

    #[test]
    fn rejects_bare_result_for_vec_u8() {
        let rt = return_type(parse_quote!(Result));
        assert!(!is_result_vec_u8(&rt));
    }

    #[test]
    fn rejects_non_result_for_vec_u8() {
        let rt = return_type(parse_quote!(Option<Vec<u8>>));
        assert!(!is_result_vec_u8(&rt));
    }

    #[test]
    fn rejects_default_return_for_vec_u8() {
        assert!(!is_result_vec_u8(&ReturnType::Default));
    }

    // --- is_result_unit ---

    #[test]
    fn accepts_result_unit() {
        let rt = return_type(parse_quote!(Result<()>));
        assert!(is_result_unit(&rt));
    }

    #[test]
    fn accepts_qualified_result_unit() {
        let rt = return_type(parse_quote!(anchor_lang::Result<()>));
        assert!(is_result_unit(&rt));
    }

    #[test]
    fn rejects_result_vec_u8_for_unit() {
        let rt = return_type(parse_quote!(Result<Vec<u8>>));
        assert!(!is_result_unit(&rt));
    }

    #[test]
    fn rejects_result_string_for_unit() {
        let rt = return_type(parse_quote!(Result<String>));
        assert!(!is_result_unit(&rt));
    }

    #[test]
    fn rejects_bare_result_for_unit() {
        let rt = return_type(parse_quote!(Result));
        assert!(!is_result_unit(&rt));
    }

    #[test]
    fn rejects_default_return_for_unit() {
        assert!(!is_result_unit(&ReturnType::Default));
    }

    #[test]
    fn rejects_non_result_for_unit() {
        let rt = return_type(parse_quote!(Option<()>));
        assert!(!is_result_unit(&rt));
    }

    // --- extract_result_inner_type ---

    #[test]
    fn extracts_unit_inner_type() {
        let rt = return_type(parse_quote!(Result<()>));
        let inner = extract_result_inner_type(&rt).unwrap();
        assert!(matches!(inner, Type::Tuple(TypeTuple { elems, .. }) if elems.is_empty()));
    }

    #[test]
    fn extracts_vec_u8_inner_type() {
        let rt = return_type(parse_quote!(Result<Vec<u8>>));
        let inner = extract_result_inner_type(&rt).unwrap();
        assert!(matches!(inner, Type::Path(TypePath { path, .. })
            if path.segments.last().is_some_and(|s| s.ident == "Vec")));
    }

    #[test]
    fn returns_none_for_bare_result() {
        let rt = return_type(parse_quote!(Result));
        assert!(extract_result_inner_type(&rt).is_none());
    }

    #[test]
    fn returns_none_for_non_result() {
        let rt = return_type(parse_quote!(Option<()>));
        assert!(extract_result_inner_type(&rt).is_none());
    }

    #[test]
    fn returns_none_for_default_return() {
        assert!(extract_result_inner_type(&ReturnType::Default).is_none());
    }

    // --- type_ends_with ---

    #[test]
    fn type_ends_with_matches_simple_ident() {
        let ty: Type = parse_quote!(OnRecvPacketMsg);
        assert!(type_ends_with(&ty, "OnRecvPacketMsg"));
    }

    #[test]
    fn type_ends_with_matches_qualified_path() {
        let ty: Type = parse_quote!(solana_ibc_types::OnRecvPacketMsg);
        assert!(type_ends_with(&ty, "OnRecvPacketMsg"));
    }

    #[test]
    fn type_ends_with_rejects_wrong_name() {
        let ty: Type = parse_quote!(OnTimeoutPacketMsg);
        assert!(!type_ends_with(&ty, "OnRecvPacketMsg"));
    }

    #[test]
    fn type_ends_with_rejects_tuple() {
        let ty: Type = parse_quote!(());
        assert!(!type_ends_with(&ty, "OnRecvPacketMsg"));
    }
}
