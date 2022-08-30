#[doc = "Register `CV0` reader"]
pub struct R(crate::R<CV0_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<CV0_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<CV0_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<CV0_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Field `CV` reader - Counter Value"]
pub type CV_R = crate::FieldReader<u32, u32>;
impl R {
    #[doc = "Bits 0:31 - Counter Value"]
    #[inline(always)]
    pub fn cv(&self) -> CV_R {
        CV_R::new(self.bits)
    }
}
#[doc = "Counter Value (channel = 0)\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [cv0](index.html) module"]
pub struct CV0_SPEC;
impl crate::RegisterSpec for CV0_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [cv0::R](R) reader structure"]
impl crate::Readable for CV0_SPEC {
    type Reader = R;
}
#[doc = "`reset()` method sets CV0 to value 0"]
impl crate::Resettable for CV0_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
