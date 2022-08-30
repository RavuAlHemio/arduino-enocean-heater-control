#[doc = "Register `QIDR` writer"]
pub struct W(crate::W<QIDR_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<QIDR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::ops::DerefMut for W {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl From<crate::W<QIDR_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<QIDR_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `IDX` writer - InDeX"]
pub type IDX_W<'a, const O: u8> = crate::BitWriter<'a, u32, QIDR_SPEC, bool, O>;
#[doc = "Field `DIRCHG` writer - DIRection CHanGe"]
pub type DIRCHG_W<'a, const O: u8> = crate::BitWriter<'a, u32, QIDR_SPEC, bool, O>;
#[doc = "Field `QERR` writer - Quadrature ERRor"]
pub type QERR_W<'a, const O: u8> = crate::BitWriter<'a, u32, QIDR_SPEC, bool, O>;
impl W {
    #[doc = "Bit 0 - InDeX"]
    #[inline(always)]
    pub fn idx(&mut self) -> IDX_W<0> {
        IDX_W::new(self)
    }
    #[doc = "Bit 1 - DIRection CHanGe"]
    #[inline(always)]
    pub fn dirchg(&mut self) -> DIRCHG_W<1> {
        DIRCHG_W::new(self)
    }
    #[doc = "Bit 2 - Quadrature ERRor"]
    #[inline(always)]
    pub fn qerr(&mut self) -> QERR_W<2> {
        QERR_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "QDEC Interrupt Disable Register\n\nThis register you can [`write_with_zero`](crate::generic::Reg::write_with_zero). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [qidr](index.html) module"]
pub struct QIDR_SPEC;
impl crate::RegisterSpec for QIDR_SPEC {
    type Ux = u32;
}
#[doc = "`write(|w| ..)` method takes [qidr::W](W) writer structure"]
impl crate::Writable for QIDR_SPEC {
    type Writer = W;
}
