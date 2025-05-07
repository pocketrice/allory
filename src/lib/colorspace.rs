use std::cmp::max;
use std::f32::consts::PI;
use std::ops::{Mul, MulAssign};
use na::{abs, Matrix3, Vector3, Vector4, Vector5};
use nalgebra::{Matrix4, Matrix4x1};

// type VideoSysConv = fn(u16, u16, u16) -> (f32, f32, f32, f32, f32);
enum VideoSystem {
    HiVision,
    HDTV,
    SDTV
}

enum HSType {
    Intensity,
    Value,
    Lightness
}
impl VideoSystem {
    // https://en.wikipedia.org/wiki/YPbPr... must convert raw component signal into gamma-corrected RGB
                                            // Y'  R'-Y'  B'-Y'  P'b  P'r
    fn v2c(&self, rv: u16, gv: u16, bv: u16, lam: f32) -> (f32, f32, f32, f32, f32) {
        let (r, g, b) = compmv2rgb(rv, gv, bv);
        self.c2c(r,g,b,lam)
    }

    fn c2c(&self, r: u16, g: u16, b: u16, lam: f32) -> (f32, f32, f32, f32, f32) {
        let (dr, dg, db) = {
            (gamma_correct(&(r as f32), &lam), gamma_correct(&(g as f32), &lam), gamma_correct(&(b as f32), &lam))
        };

        let y;
        let rmy;
        let bmy;
        let pb;
        let pr;

        match self {
            VideoSystem::HiVision => {
                y = 0.212 * dr + 0.701 * dg + 0.087 * db;       // Y'
                rmy = 0.7874 * dr - 0.7152 * dg - 0.0722 * db;  // R'-Y'
                bmy = -0.2126 * dr - 0.7152 * dg + 0.9278 * db; // B'-Y'
                pb = bmy / 1.826;
                pr = rmy / 1.576;
            }

            VideoSystem::HDTV => {
                y = 0.2126 * dr + 0.7152 * dg + 0.0722 * db;
                rmy = 0.7874 * dr - 0.7152 * dg - 0.0722 * db;
                bmy = -0.2126 * dr - 0.7152 * dg + 0.9278 * db;
                pb = (0.5 / (1.0 - 0.0722)) * bmy;
                pr = (0.5 / (1.0 - 0.2126)) * rmy;
            }

            VideoSystem::SDTV => {
                y = 0.299 * dr + 0.587 * dg + 0.114 * db;
                rmy = 0.701 * dr - 0.587 * dg - 0.114 * db;
                bmy = -0.299 * dr - 0.587 * dg + 0.886 * db;
                pb = 0.564 * bmy;
                pr = 0.713 * rmy;
            }
        }

        (y, rmy, bmy, pb, pr)
    }
}

trait ColorspaceKernel {
    fn extract(self) -> (u16, u16, u16, u16); // Convert to raw RGBA
    fn extract_prem(self) -> (u16, u16, u16, u16); // Loses precision, better with matrices. Default so cheapest!
    fn extract_rep(self) -> Matrix4x1<u16>; // Implicitly requires cmat4 rep despite https://www.reddit.com/r/rust/comments/ql4gfd/why_cant_you_have_fields_as_traits_in_rust/
    fn brighten(self, t: f32);  // Made-up word for both lightening and darkening
    fn blend(self, t: f32, other: Self);
    fn add(self, other: Self);
    fn sub(self, other: Self);
    fn to_rgb(self) -> Srgba;
}

trait ColorspaceOps { // Mostly filters, non-linear ops, n' such
    fn palette(self, other: Self, n: u8); // Generate n colors between self and other
    fn contrast(self, t: f32);
    fn saturate(self, t: f32);
    fn hueshift(self, theta: i32);

    // https://64.github.io/tonemapping/
    fn tonemap<T>(self, mapper: T) where T: FnMut(u16) -> (u16);

    // https://en.wikipedia.org/wiki/Gamma_correction
    // Gamma correction is represented by $V_out = aV_in^\lambda$
    fn gamma(self, a: f32, lambda: f32);
}

// Check out https://lisyarus.github.io/blog/posts/transforming-colors-with-matrices.html.
// Colorspace swapping based on implem of From by https://docs.rs/bevy/latest/bevy/color/struct.Srgba.html

pub struct Srgb { // sRGB (alphalocked)
    rep: Matrix4x1<u16>
}

pub struct Srgba { // sRGBA
    rep: Matrix4x1<u16>
}

pub struct HSLuv { // HSLuv
    rep: Matrix4x1<u16>
}


impl ColorspaceKernel for Srgba {
    fn extract(self) -> (u16, u16, u16, u16) {
        let alpha = self.rep.w;
        (self.rep.x / alpha, self.rep.y / alpha, self.rep.z / alpha, alpha)
    }

    fn extract_prem(self) -> (u16, u16, u16, u16) {
        (self.rep.x, self.rep.y, self.rep.z, self.rep.w)
    }

    fn extract_rep(self) -> Matrix4x1<u16> {
        self.rep
    }

    fn brighten(mut self, t: f32) { // Merge darken/lighten... negative = darken, positive = lighten.
        let mat = if t > 0.0 {
            Matrix4::new(1.0-t, 0.0, 0.0, 1.0,
                         0.0, 1.0-t, 0.0, 1.0,
                         0.0, 0.0, 1.0-t, 1.0,
                         0.0, 0.0, 0.0, 1.0);
        } else {
            let mt = -t;
            Matrix4::new(1.0-mt, 0.0, 0.0, 0.0,
                         0.0, 1.0-mt, 0.0, 0.0,
                         0.0, 0.0, 1.0-mt, 0.0,
                         0.0, 0.0, 0.0, 1.0);
        };

        self.rep.mul_assign(mat);
    }

    fn blend(mut self, t: f32, other: Self) {
        let (dr, dg, db) = (other.rep.x as f32, other.rep.y as f32, other.rep.z as f32);
        let mat = Matrix4::new(1.0-t, 0.0, 0.0, t * dr,
                                          0.0, 1.0-t, 0.0, t * dg,
                                          0.0, 0.0, 1.0-t, t * db,
                                          0.0, 0.0, 0.0, 1.0);

        self.rep.mul_to(&mat, &mut self.rep);
    }

    fn add(mut self, other: Self) {
        let (dr, dg, db) = (other.rep.x as f32, other.rep.y as f32, other.rep.z as f32);
        let mat = Matrix4::new(1.0, 0.0, 0.0, dr,
                                         0.0, 1.0, 0.0, dg,
                                         0.0, 0.0, 1.0, db,
                                         0.0, 0.0, 0.0, 1.0);

        self.rep.mul_to(&mat, &mut self.rep);
    }

    fn sub(mut self, other: Self) {
        let (dr, dg, db) = (other.rep.x as f32, other.rep.y as f32, other.rep.z as f32);
        let mat = Matrix4::new(1.0, 0.0, 0.0, -dr,
                               0.0, 1.0, 0.0, -dg,
                               0.0, 0.0, 1.0, -db,
                               0.0, 0.0, 0.0, 1.0);

        self.rep.mul_to(&mat, &mut self.rep);
    }

    fn to_rgb(self) -> Srgba { // Minutae
        self
    }
}

impl Srgba {
    pub fn to_srgb(self) -> Srgb {
        let (dr, dg, db, _) = self.extract();
        Srgb { rep: Vector4::new(dr, dg, db, 1u16) }
    }

    pub fn to_hsluv(self) -> HSLuv {
        // Plucked from https://www.hsluv.org/math/
        // (1) Convert hue to angle
        // (2) Build ray starting from (0,0) and find point where it first intersects with bounding lines
        // (3) Distance from (0,0) to this point = max chroma for given L, H.
        // (4) Scale saturation as % of this distance.


    }

    pub fn to_cmyk(self) -> CMYK { // Temporarily ignore alpha; make this a Vec5 later
        let (dr, dg, db, a) = {
            let (rr, rg, rb, a) = self.extract();
            (rr as f32 / 255.0, rg as f32 / 255.0, rb as f32 / 255.0, a)
        };

        let k = 1.0 - f32::max(f32::max(dr, dg), db);
        let ik = 1.0 - k; // efficiency!

        CMYK { rep: Vector5::new((ik - dr) / ik, (ik - dg) / ik, (ik - db) / ik, k, a as f32) }
    }

    // https://stackoverflow.com/questions/39118528/rgb-to-hsl-conversion
    // https://en.wikipedia.org/wiki/HSL_and_HSV#Hue_and_chroma
    pub fn to_hsx(self, variant: HSType) -> HSX {
        let (r,g,b) = {
            let (r,g,b,_) = self.extract();
            (r as f32, g as f32, b as f32)
        }; // These calculations work with both normalized [0-1] and reg [0-255]!!

        let (ma, mi) = {
            let buf = &[r,g,b];
            (nmax(buf), nmin(buf))
        };

        let c = ma - mi;
        assert_eq!(c, 0.0, "Violation of: HSL hue undefine if C = 0");

        let raw_h =
            if ma == r {
                ((g - b) / c) % 6.0
            } else if ma == g {
                ((b - r) / c) + 2.0
            } else {
                ((r - g) / c) + 4.0
            };
        let h = undeg(60.0 * raw_h);

        let (s,x) = match variant {
            HSType::Intensity => {
                let i = (r + g + b) * 1.0 / 3.0;
                let s = if i == 0.0 { 0.0 } else { 1.0 - mi / i };
                (s, i)
            }

            HSType::Value => {
                (if ma == 0.0 { 0.0 } else { c / ma }, ma)
            }

            HSType::Lightness => {
                let l = 0.5 * (ma + mi);
                (if l == 0.0 || l == 1.0 { 0.0 } else { c / (1.0 - (2.0 * l - 1.0).abs()) }, l)
            }
        };

        HSX { rep: Vector3::new(h,s,x), variant }
    }
    pub fn to_cielab(self) -> CIELAB {

    }

    pub fn to_ypbpr(self, vsys: VideoSystem, lam: f32) -> YPbPr {
        let (r,g,b,_) = self.extract();
        let (y, _, _, pb, pr) = vsys.c2c(r,g,b,lam);
        YPbPr { rep: Vector3::new(y, pb, pr) }
    }

    pub fn to_yiq(self, ) -> YIQ { // Original 1953 NTSC colorimetry, not 1987 SMPTE C ... https://en.wikipedia.org/wiki/YIQ
        let rgb = {
            let (r, g, b, _) = self.extract();
            Vector3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        };

        let mat = Matrix3::new(0.299, 0.587, 0.114,
                                   0.5959, -0.2746, -0.3213,
                                   0.2115, -0.5227, 0.3112);

        mat.mul_to(&rgb, &mut &rgb);
        YIQ { rep: Vector3::new(rgb.x, rgb.y, rgb.z) }
    }

    pub fn to_yuv(self, lam: f32, vsys: VideoSystem) -> YUV { // BT.470 SDTV, not BT.709 HDTV implementation https://en.wikipedia.org/wiki/Y%E2%80%B2UV
        let drgb = { // gamma-corrected!!! RGB
            let (r,g,b,_) = self.extract();
            Vector3::new(gamma_correct(&(r as f32), &lam) / 255.0, gamma_correct(&(g as f32), &lam) / 255.0, gamma_correct(&(b as f32), &lam) / 255.0)
        };

        let mat = match vsys {
            VideoSystem::SDTV => {
                Some(
                    Matrix3::new(0.299, 0.587, 0.114,
                                 -0.14713, -0.28886, 0.436,
                                 0.615, -0.51499, -0.10001)
                )
            }

            VideoSystem::HDTV => {
                Some(
                    Matrix3::new(0.2126, 0.7152, 0.0722,
                                 -0.09991, -0.33609, 0.436,
                                  0.615, -0.55861, -0.05639)
                )
            }

            _ => None
        };

        mat.unwrap().mul_to(&drgb, &mut &drgb);
        YUV { rep: Vector3::new(drgb.x, drgb.y, drgb.z) }
    }

    pub fn to_ydbdr(self) -> YDbDr {
        let rgb = {
            let (r,g,b,_) = self.extract(); // Word of warning!! All of these use RGB, not RaGaBa.
            Vector3::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        };

        let mat = Matrix3::new(0.299, 0.587, 0.114,
                                          -0.450, -0.883, 1.333,
                                          -1.333, 1.116, 0.217);

        mat.mul_to(&rgb, &mut &rgb);
        YDbDr { rep: Vector3::new(rgb.x, rgb.y, rgb.z) }
    }

    pub fn to_ycbcr(self, lam: f32, vsys: VideoSystem) -> YCbCr { // Note this is "prior to scaling/offsets to place signals in digital form" https://en.wikipedia.org/wiki/YCbCr
        let drgb = { // GC RGB
            let (r,g,b,_) = self.extract();
            Vector3::new(gamma_correct(&(r as f32), &lam) / 255.0, gamma_correct(&(g as f32), &lam) / 255.0, gamma_correct(&(b as f32), &lam) / 255.0)
        };

        let (kr, kg, kb) = match vsys { // Kr, Kg, Kb consts
            VideoSystem::SDTV => { // ITU-R BT.601
                Some(
                    (0.299, 0.587, 0.114)
                )
            }

            VideoSystem::HDTV => { // ITU-R BT.709
                Some(
                    (0.2126, 0.0722, 0.7152)
                )
            }

            _ => None
        }.unwrap();

        let mkb = 1.0 - kb; // <-- for efficiency purposes(?)
        let mkr = 1.0 - kr; // <--

        let mat = Matrix3::new(kr, kg, kb,
                                          -0.5 * kr / mkb, -0.5 * kg / mkb, 0.5,
                                          0.5, -0.5 * kg / mkr, -0.5 * kb / mkr);

        mat.mul_to(&drgb, &mut &drgb);
        YCbCr { rep: Vector3::new(drgb.x, drgb.y, drgb.z) }
    }

    pub fn to_xvycc(self, lam: f32, vsys: VideoSystem, bitdepth: u8) -> XvYcc { // "Allows negative R'G'B', expands chroma while retaining luma range, quantizes YCC values" ... https://en.wikipedia.org/wiki/XvYCC
        let (y, cb, cr) = {
            let ycc = self.to_ycbcr(lam, vsys).rep;
            (ycc.x, ycc.y, ycc.z)
        };

        let quantizer = 2.pow(bitdepth - 8); // note ... quantized to binary value since xyvcc is modern enough to be digital-only(?) but can technically store as per usual
        (quantizer * (219 * y + 16), quantizer * (224 * cb + 128), quantizer * (224 * cr + 128))
    }

    pub fn to_ciexyz(self) -> CIEXYZ { // Note that inversion should be based on the 2003 amended IEC standard. Check out the IEC 1999 standard!! https://en.wikipedia.org/wiki/SRGB#Primaries
        let rgb = { // GC RGB
            let (r,g,b,_) = self.extract();
            Vector3::new(r as f32, g as f32, b as f32)
        };

        let mat = Matrix3::new(0.4124, 0.3576, 0.1805,
                                          0.2126, 0.7152, 0.0722,
                                          0.0193, 0.1192, 0.9505);

        mat.mul_to(&rgb, &mut &rgb);
        CIEXYZ { rep: Vector3::new(rgb.x, rgb.y, rgb.z) }
    }
}

pub struct CMYK {
    rep: Vector5<f32>,
}

pub struct YPbPr {
    rep: Vector3<f32>,
}
//
pub struct XvYcc {
    rep: Vector3<u16>,
}

pub struct CIEXYZ {
    rep: Vector3<f32>,
}

pub struct HSX {
    rep: Vector3<f32>,
    variant: HSType
}
//
// pub struct HSLuv {
//     rep: Vector3<u16>,
// }
//
pub struct CIELAB {
    rep: Vector4<u16>,
}
//
// pub struct CIELUV {
//     rep: Vector3<u16>,
// }

pub struct YIQ { // NTSC color TV
    rep: Vector3<f32>
}

pub struct YUV { // PAL color TV
    rep: Vector3<f32>
}

pub struct YDbDr { // SECAM color TV
    rep: Vector3<f32>
}

pub struct YCbCr { // Digital standard color TV
    rep: Vector3<f32>
}

pub struct Oklab {
    rep: Vector3<f32>,
}

pub struct AdobeRgb {
    rep: Vector3<f32>
}

pub struct AdobeWideGamutRgb {
    rep: Vector3<f32>
}

pub struct Coloroid {

}

pub struct RYB {

}

pub struct HWB {

}

pub struct YSK {

}

pub struct TSL {

}

pub struct DciP3 {

}

pub struct YCoCg {

}

pub struct CcMmYK { // see also hexachrome

}

pub struct PCCS {

}

pub struct RG {

}

pub struct ICtCp {
    
}



// impl From<Box<dyn ColorspaceKernel>> for Srgb {
//     fn from(value: Box<dyn ColorspaceKernel>) -> Self {
//
//     }
// }

fn compmv2rgb(rv: u16, gv: u16, bv: u16) -> (u16, u16, u16) { // ⇦ component video signal; assuming 0-700 mV
    let transducer = 255.0 / 700.0;
    ((rv as f64 * transducer) as u16, (gv as f64 * transducer) as u16, (bv as f64 * transducer) as u16)
}

fn gamma_correct(vin: &f32, lambda: &f32) -> f32 {
    vin.powf(*lambda)
}

// https://www.crt-mon.com/pdf/_CRT-Data/SENCORE-TT148-Understanding-The-1VPP-Composite-Video-Signal.pdf
//fn combmv2rgb(compsig: &[u16]) -> (u16, u16, u16) {} // ⇦ combined or composite video signal

fn nmax<T: PartialOrd>(i: &[T]) -> T {
    i.iter().max()
}

fn nmin<T: PartialOrd>(i: &[T]) -> T {
    i.iter().min()
}

fn undeg(deg: f32) -> f32 {
    let mut undeg = deg;
    let range = 0.0..=360.0;

    while !range.contains(&undeg) {
        if undeg < 0.0 {
            undeg += 360.0;
        } else if undeg > 360.0 {
            undeg -= 360.0;
        }
    }

    undeg
}

fn unrad(rad: f32) -> f32 {
    let mut unrad = rad;
    let pipi = PI * 2.0;
    let range = 0.0..=pipi;

    while !range.contains(&unrad) {
        if unrad < 0.0 {
            unrad += pipi;
        } else if unrad > pipi {
            unrad -= pipi;
        }
    }

    unrad
}