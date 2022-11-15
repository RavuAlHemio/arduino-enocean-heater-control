#[doc = "Register `ECC_PR3_W8BIT` reader"]
pub struct R(crate::R<W8BIT_ECC_PR3_W8BIT_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<W8BIT_ECC_PR3_W8BIT_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<W8BIT_ECC_PR3_W8BIT_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<W8BIT_ECC_PR3_W8BIT_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Field `BITADDR` reader - Corrupted Bit Address in the Page between (i x 256) and ((i + 1) x 512) - 1) Bytes"]
pub type BITADDR_R = crate::FieldReader<u8, u8>;
#[doc = "Field `WORDADDR` reader - Corrupted Word Address in the Page between (i x 256) and ((i + 1) x 512) - 1) Bytes"]
pub type WORDADDR_R = crate::FieldReader<u8, u8>;
#[doc = "Field `NPARITY` reader - Parity N"]
pub type NPARITY_R = crate::FieldReader<u16, u16>;
impl R {
    #[doc = "Bits 0:2 - Corrupted Bit Address in the Page between (i x 256) and ((i + 1) x 512) - 1) Bytes"]
    #[inline(always)]
    pub fn bitaddr(&self) -> BITADDR_R {
        BITADDR_R::new((self.bits & 7) as u8)
    }
    #[doc = "Bits 3:10 - Corrupted Word Address in the Page between (i x 256) and ((i + 1) x 512) - 1) Bytes"]
    #[inline(always)]
    pub fn wordaddr(&self) -> WORDADDR_R {
        WORDADDR_R::new(((self.bits >> 3) & 0xff) as u8)
    }
    #[doc = "Bits 12:22 - Parity N"]
    #[inline(always)]
    pub fn nparity(&self) -> NPARITY_R {
        NPARITY_R::new(((self.bits >> 12) & 0x07ff) as u16)
    }
}
#[doc = "SMC ECC parity 3 Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [w8bit_ecc_pr3_w8bit](index.html) module"]
pub struct W8BIT_ECC_PR3_W8BIT_SPEC;
impl crate::RegisterSpec for W8BIT_ECC_PR3_W8BIT_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [w8bit_ecc_pr3_w8bit::R](R) reader structure"]
impl crate::Readable for W8BIT_ECC_PR3_W8BIT_SPEC {
    type Reader = R;
}
#[doc = "`reset()` method sets ECC_PR3_W8BIT to value 0"]
impl crate::Resettable for W8BIT_ECC_PR3_W8BIT_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
