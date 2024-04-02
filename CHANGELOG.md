# Changelog

## Unreleased

- Increased MSRV to 1.68.2
## v0.7.0 (2024-01-02)

- Allow buffer types to accept `?Sized` types
- Fix min `syn` version (v2.0.1)
- Updated `glam` to v0.25
- Updated `vek` to v0.16
- Updated `rpds` to v1
- Updated `archery` to v1

## v0.6.1 (2023-05-09)

- Fix erroring on attributes not owned by this crate

## v0.6.0 (2023-05-06)

- Inline potentially hot functions more aggressively
- Fix `clippy::extra_unused_type_parameters` warning
- Updated `syn` to v2
- Updated `glam` to v0.24
- Updated `rpds` to v0.13
- Updated `archery` to v0.5

## v0.5.0 (2023-03-04)

- Check dynamic buffer alignment is not less than 32
- Work around `trivial_bounds` error
- Increased MSRV to 1.63
- Updated `glam` to v0.23
- Updated `nalgebra` to v0.32

## v0.4.1 (2022-12-09)

- Renamed `coverage` cfg to `coverage_nightly`

## v0.4.0 (2022-11-06)

- Updated `glam` to v0.22
- Updated `rpds` to v0.12
- Updated `static-rc` to v0.6

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
