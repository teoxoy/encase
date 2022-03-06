#[cfg(feature = "archery")]
#[cfg_attr(docs, doc(cfg(feature = "archery")))]
mod archery;
#[cfg(feature = "static-rc")]
#[cfg_attr(docs, doc(cfg(feature = "static-rc")))]
mod static_rc;

#[cfg(feature = "cgmath")]
#[cfg_attr(docs, doc(cfg(feature = "cgmath")))]
mod cgmath;
#[cfg(feature = "glam")]
#[cfg_attr(docs, doc(cfg(feature = "glam")))]
mod glam;
#[cfg(feature = "mint")]
#[cfg_attr(docs, doc(cfg(feature = "mint")))]
mod mint;
#[cfg(feature = "nalgebra")]
#[cfg_attr(docs, doc(cfg(feature = "nalgebra")))]
mod nalgebra;
#[cfg(feature = "ultraviolet")]
#[cfg_attr(docs, doc(cfg(feature = "ultraviolet")))]
mod ultraviolet;
#[cfg(feature = "vek")]
#[cfg_attr(docs, doc(cfg(feature = "vek")))]
mod vek;

#[cfg(feature = "arrayvec")]
#[cfg_attr(docs, doc(cfg(feature = "arrayvec")))]
mod arrayvec;
#[cfg(feature = "ndarray")]
#[cfg_attr(docs, doc(cfg(feature = "ndarray")))]
mod ndarray;
#[cfg(feature = "smallvec")]
#[cfg_attr(docs, doc(cfg(feature = "smallvec")))]
mod smallvec;
#[cfg(feature = "tinyvec")]
#[cfg_attr(docs, doc(cfg(feature = "tinyvec")))]
mod tinyvec;

#[cfg(feature = "im")]
#[cfg_attr(docs, doc(cfg(feature = "im")))]
mod im;
#[cfg(feature = "im-rc")]
#[cfg_attr(docs, doc(cfg(feature = "im-rc")))]
mod im_rc;
#[cfg(feature = "imbl")]
#[cfg_attr(docs, doc(cfg(feature = "imbl")))]
mod imbl;
#[cfg(all(feature = "rpds", feature = "archery"))]
#[cfg_attr(docs, doc(cfg(all(feature = "rpds", feature = "archery"))))]
mod rpds;
