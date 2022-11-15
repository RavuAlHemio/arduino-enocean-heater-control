#[doc = "Register `MDL7` reader"]
pub struct R(crate::R<MDL7_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<MDL7_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<MDL7_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<MDL7_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `MDL7` writer"]
pub struct W(crate::W<MDL7_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<MDL7_SPEC>;
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
impl From<crate::W<MDL7_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<MDL7_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `MDL` reader - Message Data Low Value"]
pub type MDL_R = crate::FieldReader<u32, u32>;
#[doc = "Field `MDL` writer - Message Data Low Value"]
pub type MDL_W<'a, const O: u8> = crate::FieldWriter<'a, u32, MDL7_SPEC, u32, u32, 32, O>;
impl R {
    #[doc = "Bits 0:31 - Message Data Low Value"]
    #[inline(always)]
    pub fn mdl(&self) -> MDL_R {
        MDL_R::new(self.bits)
    }
}
impl W {
    #[doc = "Bits 0:31 - Message Data Low Value"]
    #[inline(always)]
    #[must_use]
    pub fn mdl(&mut self) -> MDL_W<0> {
        MDL_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "Mailbox Data Low Register (MB = 7)\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [mdl7](index.html) module"]
pub struct MDL7_SPEC;
impl crate::RegisterSpec for MDL7_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [mdl7::R](R) reader structure"]
impl crate::Readable for MDL7_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [mdl7::W](W) writer structure"]
impl crate::Writable for MDL7_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
#[doc = "`reset()` method sets MDL7 to value 0"]
impl crate::Resettable for MDL7_SPEC {
    const RESET_VALUE: Self::Ux = 0;
}
