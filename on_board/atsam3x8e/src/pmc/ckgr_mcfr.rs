#[doc = "Register `CKGR_MCFR` reader"]
pub struct R(crate::R<CKGR_MCFR_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<CKGR_MCFR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<CKGR_MCFR_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<CKGR_MCFR_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Field `MAINF` reader - Main Clock Frequency"]
pub type MAINF_R = crate::FieldReader<u16, u16>;
#[doc = "Field `MAINFRDY` reader - Main Clock Ready"]
pub type MAINFRDY_R = crate::BitReader<bool>;
impl R {
    #[doc = "Bits 0:15 - Main Clock Frequency"]
    #[inline(always)]
    pub fn mainf(&self) -> MAINF_R {
        MAINF_R::new((self.bits & 0xffff) as u16)
    }
    #[doc = "Bit 16 - Main Clock Ready"]
    #[inline(always)]
    pub fn mainfrdy(&self) -> MAINFRDY_R {
        MAINFRDY_R::new(((self.bits >> 16) & 1) != 0)
    }
}
#[doc = "Main Clock Frequency Register\n\nThis register you can [`read`](crate::generic::Reg::read). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [ckgr_mcfr](index.html) module"]
pub struct CKGR_MCFR_SPEC;
impl crate::RegisterSpec for CKGR_MCFR_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [ckgr_mcfr::R](R) reader structure"]
impl crate::Readable for CKGR_MCFR_SPEC {
    type Reader = R;
}
#[doc = "`reset()` method sets CKGR_MCFR to value 0"]
impl crate::Resettable for CKGR_MCFR_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
