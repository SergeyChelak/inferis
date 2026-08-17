# Asset Bundle Format

Assets stored in the compiled bundle. The data format is pretty simple and it could be described as following:

| Name        | Type              | Description                                          |
| ----------- | ----------------- | ---------------------------------------------------- |
| Asset type  | u8                | texture, animation, color, gradient, binary          |
| Id length   | u64 (LE)          | length of asset name (used to identify) in the app   |
| Id          | [u8; id length]   | asset id (name)                                      |
| Raw Type    | u8                | 0 for binary, 1 for string                           |
| Data length | u64 (LE)          | length of asset payload                              |
| Data        | [u8; data length] | asset payload (binary or string)                     |

This structure is repeated for each asset in bundle.

All multi-byte integers are stored as fixed-width little-endian values,
so a bundle built on one platform is readable on any other.