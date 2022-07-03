# Changelog

## v0.3.0 (2022-07-03)

- Renamed `Size::SIZE` to `ShaderSize::SHADER_SIZE`
- Updated `glam` to v0.21
- Increased MSRV to 1.58
- Fix `clippy::missing_const_for_fn` warning

## v0.2.1 (2022-06-14)

- Fix padding not being generated for one field structs

## v0.2.0 (2022-05-05)

- Renamed `WgslType` to `ShaderType`
- Removed `assert_uniform_compat` derive macro helper attribute
- Fixed crate not compiling on latest rustc in some scenarios
- Added ability for other crates to wrap the derive macro implementation for re-export purposes
- Updated `nalgebra` to v0.31 and `imbl` to v2

## v0.1.3 (2022-03-16)

- Improved uniform address space doc examples

## v0.1.2 (2022-03-15)

- Fixed uniform address space alignment requirements

## v0.1.1 (2022-03-09)

- Added logo
- Fixed broken links in docs

## v0.1.0 (2022-03-06)

- Initial release
