#[doc = "Register `TUND` reader"]
pub struct R(crate::R<TUND_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<TUND_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<TUND_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<TUND_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `TUND` writer"]
pub struct W(crate::W<TUND_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<TUND_SPEC>;
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
impl From<crate::W<TUND_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<TUND_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `TUND` reader - Transmit Underruns"]
pub type TUND_R = crate::FieldReader<u8, u8>;
#[doc = "Field `TUND` writer - Transmit Underruns"]
pub type TUND_W<'a, const O: u8> = crate::FieldWriter<'a, u32, TUND_SPEC, u8, u8, 8, O>;
impl R {
    #[doc = "Bits 0:7 - Transmit Underruns"]
    #[inline(always)]
    pub fn tund(&self) -> TUND_R {
        TUND_R::new((self.bits & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bits 0:7 - Transmit Underruns"]
    #[inline(always)]
    #[must_use]
    pub fn tund(&mut self) -> TUND_W<0> {
        TUND_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "Transmit Underrun Errors Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [tund](index.html) module"]
pub struct TUND_SPEC;
impl crate::RegisterSpec for TUND_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [tund::R](R) reader structure"]
impl crate::Readable for TUND_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [tund::W](W) writer structure"]
impl crate::Writable for TUND_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets TUND to value 0"]
impl crate::Resettable for TUND_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
