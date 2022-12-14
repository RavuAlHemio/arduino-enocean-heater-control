#[doc = "Register `TNCR` reader"]
pub struct R(crate::R<TNCR_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<TNCR_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<TNCR_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<TNCR_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `TNCR` writer"]
pub struct W(crate::W<TNCR_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<TNCR_SPEC>;
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
impl From<crate::W<TNCR_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<TNCR_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `TXNCTR` reader - Transmit Counter Next"]
pub type TXNCTR_R = crate::FieldReader<u16, u16>;
#[doc = "Field `TXNCTR` writer - Transmit Counter Next"]
pub type TXNCTR_W<'a, const O: u8> = crate::FieldWriter<'a, u32, TNCR_SPEC, u16, u16, 16, O>;
impl R {
    #[doc = "Bits 0:15 - Transmit Counter Next"]
    #[inline(always)]
    pub fn txnctr(&self) -> TXNCTR_R {
        TXNCTR_R::new((self.bits & 0xffff) as u16)
    }
}
impl W {
    #[doc = "Bits 0:15 - Transmit Counter Next"]
    #[inline(always)]
    #[must_use]
    pub fn txnctr(&mut self) -> TXNCTR_W<0> {
        TXNCTR_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "Transmit Next Counter Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tncr](index.html) module"]
pub struct TNCR_SPEC;
impl crate::RegisterSpec for TNCR_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [tncr::R](R) reader structure"]
impl crate::Readable for TNCR_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [tncr::W](W) writer structure"]
impl crate::Writable for TNCR_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets TNCR to value 0"]
impl crate::Resettable for TNCR_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
