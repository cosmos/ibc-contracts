# DummyLightClient

`DummyLightClient` is an insecure `ILightClient` implementation for local development and tests only. It accepts arbitrary updates and does not verify cryptographic proofs.

- `updateClient(bytes)` expects ABI encoding of `DummyLightClient.MsgUpdateClient`.
- Updates populate known consensus timestamps and explicit `(height, path) -> value hash` membership records.
- Membership succeeds when the queried `(height, path, value)` matches a stored record.
- Non-membership succeeds when no record exists for `(height, path)` at a known height.
- Proof bytes are ignored.

Do not use this client for production trust assumptions.
