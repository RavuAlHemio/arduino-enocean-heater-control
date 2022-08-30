#[doc = "Register `DEVDMANXTDSC6` reader"]
pub struct R(crate::R<DEVDMANXTDSC6_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<DEVDMANXTDSC6_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<DEVDMANXTDSC6_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<DEVDMANXTDSC6_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `DEVDMANXTDSC6` writer"]
pub struct W(crate::W<DEVDMANXTDSC6_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<DEVDMANXTDSC6_SPEC>;
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
impl From<crate::W<DEVDMANXTDSC6_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<DEVDMANXTDSC6_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `NXT_DSC_ADD` reader - Next Descriptor Address"]
pub type NXT_DSC_ADD_R = crate::FieldReader<u32, u32>;
#[doc = "Field `NXT_DSC_ADD` writer - Next Descriptor Address"]
pub type NXT_DSC_ADD_W<'a, const O: u8> =
    crate::FieldWriter<'a, u32, DEVDMANXTDSC6_SPEC, u32, u32, 32, O>;
impl R {
    #[doc = "Bits 0:31 - Next Descriptor Address"]
    #[inline(always)]
    pub fn nxt_dsc_add(&self) -> NXT_DSC_ADD_R {
        NXT_DSC_ADD_R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Next Descriptor Address"]
    #[inline(always)]
    pub fn nxt_dsc_add(&mut self) -> NXT_DSC_ADD_W<0> {
        NXT_DSC_ADD_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "Device DMA Channel Next Descriptor Address Register (n = 6)\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [devdmanxtdsc6](index.html) module"]
pub struct DEVDMANXTDSC6_SPEC;
impl crate::RegisterSpec for DEVDMANXTDSC6_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [devdmanxtdsc6::R](R) reader structure"]
impl crate::Readable for DEVDMANXTDSC6_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [devdmanxtdsc6::W](W) writer structure"]
impl crate::Writable for DEVDMANXTDSC6_SPEC {
    type Writer = W;
}
#[doc = "`reset()` method sets DEVDMANXTDSC6 to value 0"]
impl crate::Resettable for DEVDMANXTDSC6_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
