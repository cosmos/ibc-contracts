use crate::{hex_prefix::CompactPathRef, rlp::RlpItem, MptError};

#[derive(Clone, Debug)]
pub(crate) enum Node<'a> {
    Branch(RlpItem<'a>),
    Extension {
        path: CompactPathRef<'a>,
        child: RlpItem<'a>,
    },
    Leaf {
        path: CompactPathRef<'a>,
        value: &'a [u8],
    },
}

pub(crate) fn decode_node(item: RlpItem<'_>, branch_field: usize) -> Result<Node<'_>, MptError> {
    let mut cursor = item.list_cursor()?;
    let first = cursor.next()?.ok_or(MptError::InvalidNodeArity(0))?;
    let second = cursor.next()?.ok_or(MptError::InvalidNodeArity(1))?;
    let Some(third) = cursor.next()? else {
        let path = crate::hex_prefix::decode_compact_path_ref(first.as_bytes()?)?;
        return if path.is_leaf() {
            Ok(Node::Leaf {
                path,
                value: second.as_bytes()?,
            })
        } else {
            Ok(Node::Extension {
                path,
                child: second,
            })
        };
    };

    let mut selected = match branch_field {
        0 => Some(first),
        1 => Some(second),
        2 => Some(third),
        _ => None,
    };
    let mut actual = 3_usize;
    while let Some(field) = cursor.next()? {
        if actual == branch_field {
            selected = Some(field);
        }
        actual = actual.checked_add(1).ok_or(MptError::PathOverflow)?;
    }
    if actual != 17 {
        return Err(MptError::InvalidNodeArity(actual));
    }
    selected.map(Node::Branch).ok_or(MptError::PathOverflow)
}
