#[cfg(feature = "archery")]
mod archery;
#[cfg(feature = "static-rc")]
mod static_rc;

#[cfg(feature = "cgmath")]
mod cgmath;
#[cfg(feature = "glam")]
mod glam;
#[cfg(feature = "mint")]
mod mint;
#[cfg(feature = "nalgebra")]
mod nalgebra;
#[cfg(feature = "ultraviolet")]
mod ultraviolet;
#[cfg(feature = "vek")]
mod vek;

#[cfg(feature = "arrayvec")]
mod arrayvec;
#[cfg(feature = "ndarray")]
mod ndarray;
#[cfg(feature = "smallvec")]
mod smallvec;
#[cfg(feature = "tinyvec")]
mod tinyvec;

#[cfg(feature = "im")]
mod im;
#[cfg(feature = "im-rc")]
mod im_rc;
#[cfg(feature = "imbl")]
mod imbl;
#[cfg(all(feature = "rpds", feature = "archery"))]
mod rpds;
