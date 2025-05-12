// use std::any::Any;
// use std::cmp::max;
// use std::f32::consts::PI;
// use std::ops::{Div, Mul, MulAssign};
// use na::{Dim, Dyn, Dynamic, Matrix, Matrix3, Matrix3x1, OMatrix, RawStorage, Scalar, Vector3, Vector4, Vector5, U1, U4};
// use nalgebra::{Matrix4, Matrix4x1};
// use num_traits::{One, Pow};
// // type VideoSysConv = fn(u16, u16, u16) -> (f32, f32, f32, f32, f32);
//
// const CIE_DELTA: f32 = 6.0 / 29.0;
// const BRADFORD_CRD: Matrix3<f32> = Matrix3::new(0.8951000, 0.2664000, -0.1614000,
//                                               -0.750200, 1.7135000, 0.0367000,
//                                               0.0389000, -0.0685000, 1.0296000); // <-- Bradford cone response domain, [M_A]
// const BRADFORD_CRD_INV: Matrix3<f32> = Matrix3::new(0.9869929, -0.1470543, 0.1599627,
//                                                      0.4323053, 0.5183603, 0.0492912,
//                                                     -0.0085287, 0.0400428, 0.9684867); // <-- Inverse of Bradford CRD, [M_A]\ivr
//
//
// enum VideoSystem {
//     HiVision,
//     HDTV,
//     SDTV
// }
//
// enum HSType {
//     Intensity,
//     Value,
//     Lightness
// }
//
// enum StandardIlluminant {
//     A,
//     B,
//     C,
//     D50,
//     D55,
//     D65,
//     D75,
//     E,
//     F2,
//     F7,
//     F11
// }
//
// impl StandardIlluminant {
//     fn tristimuli(&self) -> (f32, f32, f32) { // Reference illuminant tristimulus values
//         match self {
//             StandardIlluminant::A => (1.09850, 1.0, 0.35585),
//             StandardIlluminant::B => (0.99072, 1.0, 0.85223),
//             StandardIlluminant::C => (0.98074, 1.0, 1.18232),
//             StandardIlluminant::D50 => (0.96422, 1.0, 0.82521),
//             StandardIlluminant::D55 => (0.95682, 1.0, 0.92149),
//             StandardIlluminant::D65 => (0.95047, 1.0, 1.08883),
//             StandardIlluminant::D75 => (0.94972, 1.0, 1.22638),
//             StandardIlluminant::E => (1.0, 1.0, 1.0),
//             StandardIlluminant::F2 => (0.99186, 1.0, 0.67393),
//             StandardIlluminant::F7 => (0.95041, 1.0, 1.08747),
//             StandardIlluminant::F11 => (1.00962, 1.0, 0.64350)
//         }
//     }
//
//     // For chromatic adaptation to different standard illuminant... http://www.brucelindbloom.com/index.html?Eqn_ChromAdapt.html
//     fn adaptation_matrix(&self, dst: StandardIlluminant) -> Matrix3<f32> { // Use Bradford over XYZ and Von Kries...
//         if self.type_id() == dst.type_id() {
//             Matrix3::identity()
//         } else {
//             let crd_src = { // <-- cone response domain ρ_s, γ_s, β_s
//                 let tristim_src = self.tristimuli();
//                 let mut wref_src = Vector3::new(tristim_src.0, tristim_src.1, tristim_src.2); // <-- reference white XYZ_WS
//                 BRADFORD_CRD.mul(wref_src)
//             };
//
//             let crd_dst = {
//                 let tristim_dst = dst.tristimuli();
//                 let mut wref_dst = Vector3::new(tristim_dst.0, tristim_dst.1, tristim_dst.2);
//                 BRADFORD_CRD.mul(wref_dst)
//             };
//
//             let crd_tf_diag = {
//                 Matrix3::new(crd_dst.x / crd_src.x, 0.0, 0.0,
//                              0.0, crd_dst.y / crd_src.y, 0.0,
//                              0.0, 0.0, crd_dst.z / crd_src.z)
//             };
//
//             BRADFORD_CRD_INV.mul(crd_tf_diag).mul(BRADFORD_CRD)
//         }
//     }
// }
//
// enum RGBWorkspaceType { // NOTE ~ this is only for conversion from linear RGB, so use Srgb::to_linear first.
//     AdobeRGB1998,
//     AppleRGB,
//     BestRGB,
//     BetaRGB,
//     BruceRGB,
//     CIERGB,
//     ColorMatchRGB,
//     DonRGB4,
//     ECIRGB,
//     EktaSpacePS5,
//     NTSCRGB,
//     PalSecamRGB,
//     ProPhotoRGB,
//     SmpteCRGB,
//     WideGamutRGB
// }
//
// impl RGBWorkspaceType {
//
//     // RGBWS -> reference white XYZ
//    fn m(&self) -> Matrix3<f32> {
//        match self {
//            RGBWorkspaceType::AdobeRGB1998 => {
//                Matrix3::new(0.5767309, 0.1855540, 0.1881852,
//                             0.2973769, 0.6273491, 0.0752741,
//                             0.0270343, 0.0706872, 0.9911085)
//            }
//            RGBWorkspaceType::AppleRGB => {
//                Matrix3::new(0.4497288, 0.3162486, 0.1844926,
//                             0.2446525, 0.6720283, 0.0833192,
//                             0.0251848, 0.1411824, 0.9224628)
//            }
//            RGBWorkspaceType::BestRGB => {
//                Matrix3::new(0.6326696, 0.2045558, 0.1269946,
//                             0.2284569, 0.7373523, 0.0341908,
//                             0.0, 0.0095142, 0.8156958)
//            }
//            RGBWorkspaceType::BetaRGB => {
//                Matrix3::new(0.6712537, 0.1745834, 0.1183829,
//                             0.3032726, 0.6637861, 0.0329413,
//                             0.0, 0.0407010, 0.7845090)
//            }
//            RGBWorkspaceType::BruceRGB => {
//                Matrix3::new(0.4674162, 0.2944512, 0.1886026,
//                             0.2410115, 0.6835475, 0.0754410,
//                             0.0219101, 0.0736128, 0.9933071)
//            }
//            RGBWorkspaceType::CIERGB => {
//                Matrix3::new(0.4887180, 0.3106803, 0.2006017,
//                             0.1762044, 0.8129847, 0.0108109,
//                             0.0, 0.0102048, 0.9897952)
//            }
//            RGBWorkspaceType::ColorMatchRGB => {
//                Matrix3::new(0.5093439, 0.3209071, 0.1339691,
//                             0.2748840, 0.6581315, 0.0669845,
//                             0.0242545, 0.1087821, 0.6921735)
//            }
//            RGBWorkspaceType::DonRGB4 => {
//                Matrix3::new(0.6457711, 0.1933511, 0.1250978,
//                             0.2783496, 0.6879702, 0.0336802,
//                             0.0037113, 0.0179861, 0.8035125)
//            }
//            RGBWorkspaceType::ECIRGB => {
//                Matrix3::new(0.6502043, 0.1780774, 0.1359384,
//                             0.3202499, 0.6020711, 0.0776791,
//                             0.0, 0.0678390, 0.7573710)
//            }
//            RGBWorkspaceType::EktaSpacePS5 => {
//                Matrix3::new(0.5938914, 0.2729801, 0.0973485,
//                             0.2606286, 0.7349465, 0.0044249,
//                             0.0, 0.0419969, 0.7832131)
//            }
//            RGBWorkspaceType::NTSCRGB => {
//                Matrix3::new(0.6068909, 0.1735011, 0.2003480,
//                             0.2989164, 0.5865990, 0.1144845,
//                             0.0, 0.0660957, 1.1162243)
//            }
//            RGBWorkspaceType::PalSecamRGB => {
//                Matrix3::new(0.4306190, 0.3415419, 0.1783091,
//                             0.2220379, 0.7066384, 0.0713236,
//                             0.0201853, 0.1295504, 0.9390944)
//            }
//            RGBWorkspaceType::ProPhotoRGB => {
//                Matrix3::new(0.7976749, 0.1351917, 0.0313534,
//                             0.2880402, 0.7118741, 0.0000857,
//                             0.0, 0.0, 0.8252100)
//            }
//            RGBWorkspaceType::SmpteCRGB => {
//                Matrix3::new(0.3935891, 0.3652497, 0.1916313,
//                             0.2124132, 0.7010437, 0.0865432,
//                             0.0187423, 0.1119313, 0.9581563)
//            }
//            RGBWorkspaceType::WideGamutRGB => {
//                Matrix3::new(0.7161046, 0.1009296, 0.1471858,
//                             0.2581874, 0.7249378, 0.0168748,
//                             0.0, 0.0517813, 0.7734287)
//            }
//        }
//    }
//
//     // Reference white XYZ -> RGBWS
//     fn m_inv(&self) -> Matrix3<f32> {
//         match self {
//             RGBWorkspaceType::AdobeRGB1998 => {
//                 Matrix3::new(2.0413690, -0.5649464, -0.3446944,
//                              -0.9692660, 1.8760108, 0.0415560,
//                              0.0134474, -0.1183897, 1.0154096)
//             }
//             RGBWorkspaceType::AppleRGB => {
//                 Matrix3::new(2.9515373, -1.2894116, -0.4738445,
//                              -1.0851093, 1.9908566, 0.0372026,
//                              0.0854934, -0.2694964, 1.0912975)
//             }
//             RGBWorkspaceType::BestRGB => {
//                 Matrix3::new(1.7552599, -0.4836786, -0.2530000,
//                              -0.5441336, 1.5068789, 0.0215528,
//                              0.0063467, -0.0175761, 1.2256959)
//             }
//             RGBWorkspaceType::BetaRGB => {
//                 Matrix3::new(1.6832270, -0.4282363, -0.2360185,
//                              -0.7710229, 1.7065571, 0.0446900,
//                              0.0400013, -0.0885376, 1.2723640)
//             }
//             RGBWorkspaceType::BruceRGB => {
//                 Matrix3::new(2.7454669, -1.1358136, -0.4350269,
//                              -0.9692660, 1.8760108, 0.0415560,
//                              0.0112723, -0.1139754, 1.0132541)
//             }
//             RGBWorkspaceType::CIERGB => {
//                 Matrix3::new(2.3706743, -0.9000405, -0.4706338,
//                              -0.5138850, 1.4253036, 0.0885814,
//                              0.0052982, -0.0146949, 1.0093968)
//             }
//             RGBWorkspaceType::ColorMatchRGB => {
//                 Matrix3::new(2.6422874, -1.2234270, -0.3930143,
//                              -1.1119763, 2.0590183, 0.0159614,
//                              0.0821699, -0.2807254, 1.4559877)
//             }
//             RGBWorkspaceType::DonRGB4 => {
//                 Matrix3::new(1.7603902, -0.4881198, -0.2536126,
//                              -0.7126288, 1.6527432, 0.0416715,
//                              0.0078207, -0.0347411, 1.2447743)
//             }
//             RGBWorkspaceType::ECIRGB => {
//                 Matrix3::new(1.7827618, -0.4969847, -0.2690101,
//                              -0.9593623, 1.9477962, -0.0275807,
//                              0.0859317, -0.1744674, 1.3228273)
//             }
//             RGBWorkspaceType::EktaSpacePS5 => {
//                 Matrix3::new(2.0043819, -0.7304844, -0.2450052,
//                              -0.7110285, 1.6202126, 0.0792227,
//                              0.0381263, -0.0868780, 1.2725438)
//             }
//             RGBWorkspaceType::NTSCRGB => {
//                 Matrix3::new(1.9099961, -0.5324542, -0.2882091,
//                              -0.9846663, 1.9991710, -0.0283082,
//                              0.0583056, -0.1183781, 0.8975535)
//             }
//             RGBWorkspaceType::PalSecamRGB => {
//                 Matrix3::new(3.0628971, -1.3931791, -0.4757517,
//                              0.9692660, 1.8760108, 0.0415560,
//                              0.0678775, -0.2288547, 1.0693490)
//             }
//             RGBWorkspaceType::ProPhotoRGB => {
//                 Matrix3::new(1.3459433, -0.2556075, -0.0511118,
//                              -0.5445989, 1.5081673, 0.0205351,
//                              0.0, 0.0, 1.2118128)
//             }
//             RGBWorkspaceType::SmpteCRGB => {
//                 Matrix3::new(3.5053960, -1.7394894, -0.5439640,
//                              -1.0690722, 1.9778245, 0.0351722,
//                              0.0563200, -0.1970226, 1.0502026)
//             }
//             RGBWorkspaceType::WideGamutRGB => {
//                 Matrix3::new(1.4628067, -0.1840623, -0.2743606,
//                              -0.5217933, 1.4472381, 0.0677227,
//                              0.0349342, -0.0968930, 1.2884099)
//             }
//         }
//     }
// }
//
// impl VideoSystem {
//     // https://en.wikipedia.org/wiki/YPbPr... must convert raw component signal into gamma-corrected RGB
//                                             // Y'  R'-Y'  B'-Y'  P'b  P'r
//     fn v2c(&self, rv: u16, gv: u16, bv: u16, lam: f32) -> (f32, f32, f32, f32, f32) {
//         let (r, g, b) = compmv2rgb(rv, gv, bv);
//         self.c2c(r,g,b,lam)
//     }
//
//     fn c2c(&self, r: u16, g: u16, b: u16, lam: f32) -> (f32, f32, f32, f32, f32) {
//         let (dr, dg, db) = {
//             (gamma_correct(&(r as f32), &lam), gamma_correct(&(g as f32), &lam), gamma_correct(&(b as f32), &lam))
//         };
//
//         let y;
//         let rmy;
//         let bmy;
//         let pb;
//         let pr;
//
//         match self {
//             VideoSystem::HiVision => {
//                 y = 0.212 * dr + 0.701 * dg + 0.087 * db;       // Y'
//                 rmy = 0.7874 * dr - 0.7152 * dg - 0.0722 * db;  // R'-Y'
//                 bmy = -0.2126 * dr - 0.7152 * dg + 0.9278 * db; // B'-Y'
//                 pb = bmy / 1.826;
//                 pr = rmy / 1.576;
//             }
//
//             VideoSystem::HDTV => {
//                 y = 0.2126 * dr + 0.7152 * dg + 0.0722 * db;
//                 rmy = 0.7874 * dr - 0.7152 * dg - 0.0722 * db;
//                 bmy = -0.2126 * dr - 0.7152 * dg + 0.9278 * db;
//                 pb = (0.5 / (1.0 - 0.0722)) * bmy;
//                 pr = (0.5 / (1.0 - 0.2126)) * rmy;
//             }
//
//             VideoSystem::SDTV => {
//                 y = 0.299 * dr + 0.587 * dg + 0.114 * db;
//                 rmy = 0.701 * dr - 0.587 * dg - 0.114 * db;
//                 bmy = -0.299 * dr - 0.587 * dg + 0.886 * db;
//                 pb = 0.564 * bmy;
//                 pr = 0.713 * rmy;
//             }
//         }
//
//         (y, rmy, bmy, pb, pr)
//     }
// }
//
// trait ColorspaceKernel {
//     fn extract(self) -> (u16, u16, u16, u16); // Convert to raw RGBA
//     fn extract_prem(self) -> (u16, u16, u16, u16); // Loses precision, better with matrices. Default so cheapest!
//     fn extract_rep(self) -> Matrix4x1<u16>; // Implicitly requires cmat4 rep despite https://www.reddit.com/r/rust/comments/ql4gfd/why_cant_you_have_fields_as_traits_in_rust/
//     fn brighten(self, t: f32);  // Made-up word for both lightening and darkening
//     fn blend(self, t: f32, other: Self);
//     fn add(self, other: Self);
//     fn sub(self, other: Self);
//     fn gamma_correct(self, lam: f32) -> f32;
//     fn to_rgb(self) -> Srgba;
//     fn inv_compand(self) -> LinearRGB;
// }
//
// trait ColorspaceOps { // Mostly filters, non-linear ops, n' such
//     fn palette(self, other: Self, n: u8); // Generate n colors between self and other
//     fn contrast(self, t: f32);
//     fn saturate(self, t: f32);
//     fn hueshift(self, theta: i32);
//
//     // https://64.github.io/tonemapping/
//     fn tonemap<T>(self, mapper: T) where T: FnMut(u16) -> (u16);
//
//     // https://en.wikipedia.org/wiki/Gamma_correction
//     // Gamma correction is represented by $V_out = aV_in^\lambda$
//     fn gamma(self, a: f32, lambda: f32);
// }
//
// // Check out https://lisyarus.github.io/blog/posts/transforming-colors-with-matrices.html.
// // Colorspace swapping based on implem of From by https://docs.rs/bevy/latest/bevy/color/struct.Srgba.html
//
// pub struct Srgb { // sRGB (alphalocked)
//     rep: Matrix4x1<u16>
// }
//
// pub struct Srgba { // sRGBA
//     rep: Matrix4x1<u16>
// }
//
// pub struct HSLuv { // HSLuv
//     rep: Matrix4x1<u16>
// }
//
//
// impl ColorspaceKernel for Srgba {
//     fn extract(self) -> (u16, u16, u16, u16) {
//         let alpha = self.rep.w;
//         (self.rep.x / alpha, self.rep.y / alpha, self.rep.z / alpha, alpha)
//     }
//
//     fn extract_prem(self) -> (u16, u16, u16, u16) {
//         (self.rep.x, self.rep.y, self.rep.z, self.rep.w)
//     }
//
//     fn extract_rep(self) -> Matrix4x1<u16> {
//         self.rep
//     }
//
//     fn brighten(mut self, t: f32) { // Merge darken/lighten... negative = darken, positive = lighten.
//         let mat = if t > 0.0 {
//             Matrix4::new(1.0-t, 0.0, 0.0, 1.0,
//                          0.0, 1.0-t, 0.0, 1.0,
//                          0.0, 0.0, 1.0-t, 1.0,
//                          0.0, 0.0, 0.0, 1.0)
//         } else {
//             let mt = -t;
//             Matrix4::new(1.0-mt, 0.0, 0.0, 0.0,
//                          0.0, 1.0-mt, 0.0, 0.0,
//                          0.0, 0.0, 1.0-mt, 0.0,
//                          0.0, 0.0, 0.0, 1.0)
//         };
//
//         let mut repcpy: OMatrix<f32, U4, U1> = self.rep.map(|m| m as f32);
//         mat.mul_to(&repcpy, &mut repcpy);
//         self.rep = repcpy.map(|m| m as u16);
//     }
//
//     fn blend(mut self, t: f32, other: Self) {
//         let (dr, dg, db) = (other.rep.x as f32, other.rep.y as f32, other.rep.z as f32);
//         let mat = Matrix4::new(1.0-t, 0.0, 0.0, t * dr,
//                                           0.0, 1.0-t, 0.0, t * dg,
//                                           0.0, 0.0, 1.0-t, t * db,
//                                           0.0, 0.0, 0.0, 1.0);
//
//         let mut repcpy: OMatrix<f32, U4, U1> = self.rep.map(|m| m as f32);
//         mat.mul_to(&repcpy, &mut repcpy);
//         self.rep = repcpy.map(|m| m as u16);
//     }
//
//     fn add(mut self, other: Self) {
//         let (dr, dg, db) = (other.rep.x as f32, other.rep.y as f32, other.rep.z as f32);
//         let mat = Matrix4::new(1.0, 0.0, 0.0, dr,
//                                          0.0, 1.0, 0.0, dg,
//                                          0.0, 0.0, 1.0, db,
//                                          0.0, 0.0, 0.0, 1.0);
//
//
//         let mut repcpy: OMatrix<f32, U4, U1> = self.rep.map(|m| m as f32);
//         mat.mul_to(&repcpy, &mut repcpy);
//         self.rep = repcpy.map(|m| m as u16);
//     }
//
//     fn sub(mut self, other: Self) {
//         let (dr, dg, db) = (other.rep.x as f32, other.rep.y as f32, other.rep.z as f32);
//         let mat = Matrix4::new(1.0, 0.0, 0.0, -dr,
//                                0.0, 1.0, 0.0, -dg,
//                                0.0, 0.0, 1.0, -db,
//                                0.0, 0.0, 0.0, 1.0);
//
//         let mut repcpy: OMatrix<f32, U4, U1> = self.rep.map(|m| m as f32);
//         mat.mul_to(&repcpy, &mut repcpy);
//         self.rep = repcpy.map(|m| m as u16);
//     }
//
//     fn to_rgb(self) -> Srgba { // Minutae
//         self
//     }
//
//     fn inv_compand(self) -> LinearRGB {
//         let (r,g,b) = {
//             let (r,g,b,_) = self.extract();
//             (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
//         };
//
//         fn s2l(v: f32) -> f32 {
//             if v <= 0.04045 {
//                 v / 12.92
//             } else {
//                 ((v + 0.055) / 1.055).powf(2.4)
//             }
//         }
//
//         LinearRGB { rep: Vector3::new(s2l(r), s2l(g), s2l(b)) }
//     }
//
//     fn gamma_correct(self, lam: f32) -> f32 {
//         self.rep.x = gamma_correct(&self.rep.x, &lam);
//         self.rep.y = gamma_correct(&self.rep.y, &lam);
//         self.rep.z = gamma_correct(&self.rep.z, &lam);
//     }
// }
//
// impl Srgba {
//     pub fn to_srgb(self) -> Srgb {
//         let (dr, dg, db, _) = self.extract();
//         Srgb { rep: Vector4::new(dr, dg, db, 1u16) }
//     }
//
//     pub fn to_hsluv(self) -> HSLuv {
//         // Plucked from https://www.hsluv.org/math/
//         // (1) Convert hue to angle
//         // (2) Build ray starting from (0,0) and find point where it first intersects with bounding lines
//         // (3) Distance from (0,0) to this point = max chroma for given L, H.
//         // (4) Scale saturation as % of this distance.
//
//
//     }
//
//     pub fn to_cmyk(self) -> CMYK { // Temporarily ignore alpha; make this a Vec5 later
//         let (dr, dg, db, a) = {
//             let (rr, rg, rb, a) = self.extract();
//             (rr as f32 / 255.0, rg as f32 / 255.0, rb as f32 / 255.0, a)
//         };
//
//         let k = 1.0 - f32::max(f32::max(dr, dg), db);
//         let ik = 1.0 - k; // efficiency!
//
//         CMYK { rep: Vector5::new((ik - dr) / ik, (ik - dg) / ik, (ik - db) / ik, k, a as f32) }
//     }
//
//     // https://stackoverflow.com/questions/39118528/rgb-to-hsl-conversion
//     // https://en.wikipedia.org/wiki/HSL_and_HSV#Hue_and_chroma
//     pub fn to_hsx(self, variant: HSType) -> HSX {
//         let (r,g,b) = {
//             let (r,g,b,_) = self.extract();
//             (r as f32, g as f32, b as f32)
//         }; // These calculations work with both normalized [0-1] and reg [0-255]!!
//
//         let (ma, mi) = {
//             let buf = &[r,g,b];
//             (nmax(buf), nmin(buf))
//         };
//
//         let c = ma - mi;
//         assert_eq!(c, 0.0, "Violation of: HSL hue undefine if C = 0");
//
//         let raw_h =
//             if ma == r {
//                 ((g - b) / c) % 6.0
//             } else if ma == g {
//                 ((b - r) / c) + 2.0
//             } else {
//                 ((r - g) / c) + 4.0
//             };
//         let h = undeg(60.0 * raw_h);
//
//         let (s,x) = match variant {
//             HSType::Intensity => {
//                 let i = (r + g + b) * 1.0 / 3.0;
//                 let s = if i == 0.0 { 0.0 } else { 1.0 - mi / i };
//                 (s, i)
//             }
//
//             HSType::Value => {
//                 (if ma == 0.0 { 0.0 } else { c / ma }, ma)
//             }
//
//             HSType::Lightness => {
//                 let l = 0.5 * (ma + mi);
//                 (if l == 0.0 || l == 1.0 { 0.0 } else { c / (1.0 - (2.0 * l - 1.0).abs()) }, l)
//             }
//         };
//
//         HSX { rep: Vector3::new(h,s,x), variant }
//     }
//
//     pub fn to_ypbpr(self, vsys: VideoSystem, lam: f32) -> YPbPr {
//         let (r,g,b,_) = self.extract();
//         let (y, _, _, pb, pr) = vsys.c2c(r,g,b,lam);
//         YPbPr { rep: Vector3::new(y, pb, pr) }
//     }
//
//     pub fn to_yiq(self) -> YIQ { // Original 1953 NTSC colorimetry, not 1987 SMPTE C ... https://en.wikipedia.org/wiki/YIQ
//         let rgb = {
//             let (r, g, b, _) = self.extract();
//             Vector3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
//         };
//
//         let mat = Matrix3::new(0.299, 0.587, 0.114,
//                                    0.5959, -0.2746, -0.3213,
//                                    0.2115, -0.5227, 0.3112);
//
//         mat.mul_to(&rgb, &mut &rgb);
//         YIQ { rep: rgb }
//     }
//
//     pub fn to_yuv(self, lam: f32, vsys: VideoSystem) -> YUV { // BT.470 SDTV, not BT.709 HDTV implementation https://en.wikipedia.org/wiki/Y%E2%80%B2UV
//         let drgb = { // gamma-corrected!!! RGB
//             let (r,g,b,_) = self.extract();
//             Vector3::new(gamma_correct(&(r as f32), &lam) / 255.0, gamma_correct(&(g as f32), &lam) / 255.0, gamma_correct(&(b as f32), &lam) / 255.0)
//         };
//
//         let mat = match vsys {
//             VideoSystem::SDTV => {
//                 Some(
//                     Matrix3::new(0.299, 0.587, 0.114,
//                                  -0.14713, -0.28886, 0.436,
//                                  0.615, -0.51499, -0.10001)
//                 )
//             }
//
//             VideoSystem::HDTV => {
//                 Some(
//                     Matrix3::new(0.2126, 0.7152, 0.0722,
//                                  -0.09991, -0.33609, 0.436,
//                                   0.615, -0.55861, -0.05639)
//                 )
//             }
//
//             _ => None
//         };
//
//         mat.unwrap().mul_to(&drgb, &mut &drgb);
//         YUV { rep: drgb }
//     }
//
//     pub fn to_ydbdr(self) -> YDbDr {
//         let rgb = {
//             let (r,g,b,_) = self.extract(); // Word of warning!! All of these use RGB, not RaGaBa.
//             Vector3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
//         };
//
//         let mat = Matrix3::new(0.299, 0.587, 0.114,
//                                           -0.450, -0.883, 1.333,
//                                           -1.333, 1.116, 0.217);
//
//         mat.mul_to(&rgb, &mut &rgb);
//         YDbDr { rep: rgb }
//     }
//
//     pub fn to_ycbcr(self, lam: f32, vsys: VideoSystem) -> YCbCr { // Note this is "prior to scaling/offsets to place signals in digital form" https://en.wikipedia.org/wiki/YCbCr
//         let drgb = { // GC RGB
//             let (r,g,b,_) = self.extract();
//             Vector3::new(gamma_correct(&(r as f32), &lam) / 255.0, gamma_correct(&(g as f32), &lam) / 255.0, gamma_correct(&(b as f32), &lam) / 255.0)
//         };
//
//         let (kr, kg, kb) = match vsys { // Kr, Kg, Kb consts
//             VideoSystem::SDTV => { // ITU-R BT.601
//                 Some(
//                     (0.299, 0.587, 0.114)
//                 )
//             }
//
//             VideoSystem::HDTV => { // ITU-R BT.709
//                 Some(
//                     (0.2126, 0.0722, 0.7152)
//                 )
//             }
//
//             _ => None
//         }.unwrap();
//
//         let mkb = 1.0 - kb; // <-- for efficiency purposes(?)
//         let mkr = 1.0 - kr; // <--
//
//         let mat = Matrix3::new(kr, kg, kb,
//                                           -0.5 * kr / mkb, -0.5 * kg / mkb, 0.5,
//                                           0.5, -0.5 * kg / mkr, -0.5 * kb / mkr);
//
//         mat.mul_to(&drgb, &mut &drgb);
//         YCbCr { rep: drgb }
//     }
//
//     pub fn to_xvycc(self, lam: f32, vsys: VideoSystem, bitdepth: u8) -> XvYcc { // "Allows negative R'G'B', expands chroma while retaining luma range, quantizes YCC values" ... https://en.wikipedia.org/wiki/XvYCC
//         let (y, cb, cr) = {
//             let ycc = self.to_ycbcr(lam, vsys).rep;
//             (ycc.x, ycc.y, ycc.z)
//         };
//
//         let quantizer = 2.pow(bitdepth - 8); // note ... quantized to binary value since xyvcc is modern enough to be digital-only(?) but can technically store as per usual
//         (quantizer * (219 * y + 16), quantizer * (224 * cb + 128), quantizer * (224 * cr + 128))
//     }
//
//     pub fn to_ciexyz(self) -> CIEXYZ { // Note that inversion should be based on the 2003 amended IEC standard. Check out the IEC 1999 standard!! https://en.wikipedia.org/wiki/SRGB#Primaries
//         let rgb = self.inv_compand().rep;
//
//         let mat = Matrix3::new(0.4124564, 0.3575761, 0.1804375,
//                                           0.2126729, 0.7151522, 0.0721750,
//                                           0.0193339, 0.1191920, 0.9503041);
//
//         mat.mul_to(&rgb, &mut &rgb);
//         CIEXYZ { rep: rgb }
//     }
//
//     pub fn to_cielab(self, arefi: StandardIlluminant) -> CIELAB {
//         let (x,y,z) = {
//             let ciexyz = self.to_ciexyz().rep;
//             (ciexyz.x, ciexyz.y, ciexyz.z)
//         };
//
//         fn f(t: f32) -> f32 {
//             let thirdth = 1.0 / 3.0;
//
//             if t > CIE_DELTA.pow(3) {
//                 t.powf(thirdth)
//             } else {
//                 thirdth * t * CIE_DELTA.powf(-2.0) + 4.0 / 29.0
//             }
//         }
//
//         let (xn, yn, zn) = arefi.value();
//
//         CIELAB { rep: Vector3::new(116.0 * f(y / yn) - 16.0,500.0 * (f(x / xn) - f(y / yn)), 200.0 * (f(y / yn) - f(z / zn))) }
//     }
//
//     pub fn to_cielch(self, arefi: StandardIlluminant) -> CIELCh {
//         let (l,a,b) = {
//             let cielab = self.to_cielab(arefi).rep;
//             (cielab.x, cielab.y, cielab.z)
//         };
//
//         CIELCh { rep: Vector3::new(l, f32::pow(a.pow(2) + b.pow(2), 0.5), (b / a).atan()) }
//     }
//
//     pub fn to_oklch(self, arefi: StandardIlluminant) -> OkLCh {
//         let (l,a,b) = {
//             let cielab = self.to_cielab(arefi).rep;
//             (cielab.x, cielab.y, cielab.z)
//         };
//
//         OkLCh { rep: Vector3::new(l, f32::pow(a.pow(2) + b.pow(2), 0.5), b.atan2(a)) }
//     }
//
//     pub fn to_oklab(self) -> Oklab { // Note this only uses Standard Illuminant D65
//         let rgb = {
//             let (r,g,b,_) = self.extract(); // Word of warning!! All of these use RGB, not RaGaBa.
//             Vector3::new(r as f32, g as f32, b as f32)
//         };
//
//         let mat1 = Matrix3::new(0.4122214708, 0.5363325363, 0.0514459929,
//                                           0.2119034982, 0.6806995451, 0.1073969566,
//                                           0.0883024619, 0.2817188376, 0.6299787005); // Note this is the (linear) sRGB mapper M1 and not the #Conversion_from_CIE_XYZ M1.
//
//         let mat2 = Matrix3::new(0.2104542553, 0.7936177850, -0.0040720468,
//                                            1.9779984951, -2.4285922050, 0.4505937099,
//                                            0.0259040371, 0.7827717662, -0.8086757660);
//
//         mat1.mul_to(&rgb, &mut &rgb);
//         let lms = rgb.map(|x| x.powf(1.0 / 3.0));
//         mat2.mul_to(&lms, &mut &lms);
//         Oklab { rep: lms }
//     }
//
//     pub fn to_rgb_workspace(self, rgbws: RGBWorkspaceType) -> RGBWorkspace {
//         // It seems that many RGBWS employ different gamma correction, such as Adobe RGB '98 using pure power 2.2... this won't apply it.
//         let xyz = self.to_ciexyz().rep;
//         let m_inv = rgbws.m_inv();
//
//         m_inv.mul_to(&xyz, &mut &xyz);
//         RGBWorkspace { rep: xyz, variant: rgbws }
//     }
//
//     pub fn to_adobe(self) -> RGBWorkspace { // <-- special function for specifically getting Adobe '98 RGB as it's well-used
//         let mut rgb = self.to_rgb_workspace(RGBWorkspaceType::AdobeRGB1998);
//         rgb.gamma_correct(2.4);
//         rgb
//     }
// }
//
// pub struct CMYK {
//     rep: Vector5<f32>,
// }
//
// pub struct YPbPr {
//     rep: Vector3<f32>,
// }
// //
// pub struct XvYcc {
//     rep: Vector3<u16>,
// }
//
// pub struct CIEXYZ {
//     rep: Vector3<f32>,
// }
//
// pub struct HSX {
//     rep: Vector3<f32>,
//     variant: HSType
// }
//
// pub struct LinearRGB { // Quintessential, most basic color representation. http://www.brucelindbloom.com/index.html?Eqn_RGB_to_XYZ.html
//     rep: Vector3<f32>
// }
//
// pub struct RGBWorkspace {
//     rep: Vector3<f32>,
//     variant: RGBWorkspaceType
// }
//
// impl RGBWorkspace {
//     fn gamma_correct(&mut self, lam: f32) {
//         self.rep.x = gamma_correct(&self.rep.x, &lam);
//         self.rep.y = gamma_correct(&self.rep.y, &lam);
//         self.rep.z = gamma_correct(&self.rep.z, &lam);
//     }
// }
//
// //
// // pub struct HSLuv {
// //     rep: Vector3<u16>,
// // }
// //
//
// pub struct OkLCh {
//     rep: Vector3<f32>
// }
//
// pub struct CIELAB {
//     rep: Vector3<f32>,
// }
//
// pub struct CIELCh {
//     rep: Vector3<f32>
// }
// //
// // pub struct CIELUV {
// //     rep: Vector3<u16>,
// // }
//
// pub struct YIQ { // NTSC color TV
//     rep: Vector3<f32>
// }
//
// pub struct YUV { // PAL color TV
//     rep: Vector3<f32>
// }
//
// pub struct YDbDr { // SECAM color TV
//     rep: Vector3<f32>
// }
//
// pub struct YCbCr { // Digital standard color TV
//     rep: Vector3<f32>
// }
//
// pub struct Oklab {
//     rep: Vector3<f32>,
// }
//
// pub struct Coloroid {
//
// }
//
// pub struct RYB {
//
// }
//
// pub struct HWB {
//
// }
//
// pub struct YSK {
//
// }
//
// pub struct TSL {
//
// }
//
// pub struct DciP3 {
//
// }
//
// pub struct YCoCg {
//
// }
//
// pub struct CcMmYK { // see also hexachrome
//
// }
//
// pub struct PCCS {
//
// }
//
// pub struct RG {
//
// }
//
// pub struct ICtCp {
//
// }
//
//
//
// // impl From<Box<dyn ColorspaceKernel>> for Srgb {
// //     fn from(value: Box<dyn ColorspaceKernel>) -> Self {
// //
// //     }
// // }
//
// fn compmv2rgb(rv: u16, gv: u16, bv: u16) -> (u16, u16, u16) { // ⇦ component video signal; assuming 0-700 mV
//     let transducer = 255.0 / 700.0;
//     ((rv as f64 * transducer) as u16, (gv as f64 * transducer) as u16, (bv as f64 * transducer) as u16)
// }
//
// fn gamma_correct(vin: &f32, lambda: &f32) -> f32 {
//     vin.powf(*lambda)
// }
//
// // https://www.crt-mon.com/pdf/_CRT-Data/SENCORE-TT148-Understanding-The-1VPP-Composite-Video-Signal.pdf
// //fn combmv2rgb(compsig: &[u16]) -> (u16, u16, u16) {} // ⇦ combined or composite video signal
//
// fn nmax<T: PartialOrd>(i: &[T]) -> T {
//     i.iter().max()
// }
//
// fn nmin<T: PartialOrd>(i: &[T]) -> T {
//     i.iter().min()
// }
//
// fn undeg(deg: f32) -> f32 {
//     let mut undeg = deg;
//     let range = 0.0..=360.0;
//
//     while !range.contains(&undeg) {
//         if undeg < 0.0 {
//             undeg += 360.0;
//         } else if undeg > 360.0 {
//             undeg -= 360.0;
//         }
//     }
//
//     undeg
// }
//
// fn unrad(rad: f32) -> f32 {
//     let mut unrad = rad;
//     let pipi = PI * 2.0;
//     let range = 0.0..=pipi;
//
//     while !range.contains(&unrad) {
//         if unrad < 0.0 {
//             unrad += pipi;
//         } else if unrad > pipi {
//             unrad -= pipi;
//         }
//     }
//
//     unrad
// }
//
// // fn mat_flip<T,R,C,S>(mat: &mut Matrix<T, R, C, S>) where
// //     T: Scalar + Div<Output = T> + One + Copy,
// //     R: Dim,
// //     C: Dim,
// //     S: RawStorage<T,R,C>
// // {
// //     let one = T::one();
// //     for m in mat.iter_mut() {
// //     }
// // }
// //
// // fn mat_hadamard