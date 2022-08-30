#[doc = "Register `OOV` reader"]
pub struct R(crate::R<OOV_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<OOV_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<OOV_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<OOV_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `OOV` writer"]
pub struct W(crate::W<OOV_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<OOV_SPEC>;
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
impl From<crate::W<OOV_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<OOV_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `OOVH0` reader - Output Override Value for PWMH output of the channel 0"]
pub type OOVH0_R = crate::BitReader<bool>;
#[doc = "Field `OOVH0` writer - Output Override Value for PWMH output of the channel 0"]
pub type OOVH0_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVH1` reader - Output Override Value for PWMH output of the channel 1"]
pub type OOVH1_R = crate::BitReader<bool>;
#[doc = "Field `OOVH1` writer - Output Override Value for PWMH output of the channel 1"]
pub type OOVH1_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVH2` reader - Output Override Value for PWMH output of the channel 2"]
pub type OOVH2_R = crate::BitReader<bool>;
#[doc = "Field `OOVH2` writer - Output Override Value for PWMH output of the channel 2"]
pub type OOVH2_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVH3` reader - Output Override Value for PWMH output of the channel 3"]
pub type OOVH3_R = crate::BitReader<bool>;
#[doc = "Field `OOVH3` writer - Output Override Value for PWMH output of the channel 3"]
pub type OOVH3_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVH4` reader - Output Override Value for PWMH output of the channel 4"]
pub type OOVH4_R = crate::BitReader<bool>;
#[doc = "Field `OOVH4` writer - Output Override Value for PWMH output of the channel 4"]
pub type OOVH4_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVH5` reader - Output Override Value for PWMH output of the channel 5"]
pub type OOVH5_R = crate::BitReader<bool>;
#[doc = "Field `OOVH5` writer - Output Override Value for PWMH output of the channel 5"]
pub type OOVH5_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVH6` reader - Output Override Value for PWMH output of the channel 6"]
pub type OOVH6_R = crate::BitReader<bool>;
#[doc = "Field `OOVH6` writer - Output Override Value for PWMH output of the channel 6"]
pub type OOVH6_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVH7` reader - Output Override Value for PWMH output of the channel 7"]
pub type OOVH7_R = crate::BitReader<bool>;
#[doc = "Field `OOVH7` writer - Output Override Value for PWMH output of the channel 7"]
pub type OOVH7_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVL0` reader - Output Override Value for PWML output of the channel 0"]
pub type OOVL0_R = crate::BitReader<bool>;
#[doc = "Field `OOVL0` writer - Output Override Value for PWML output of the channel 0"]
pub type OOVL0_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVL1` reader - Output Override Value for PWML output of the channel 1"]
pub type OOVL1_R = crate::BitReader<bool>;
#[doc = "Field `OOVL1` writer - Output Override Value for PWML output of the channel 1"]
pub type OOVL1_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVL2` reader - Output Override Value for PWML output of the channel 2"]
pub type OOVL2_R = crate::BitReader<bool>;
#[doc = "Field `OOVL2` writer - Output Override Value for PWML output of the channel 2"]
pub type OOVL2_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVL3` reader - Output Override Value for PWML output of the channel 3"]
pub type OOVL3_R = crate::BitReader<bool>;
#[doc = "Field `OOVL3` writer - Output Override Value for PWML output of the channel 3"]
pub type OOVL3_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVL4` reader - Output Override Value for PWML output of the channel 4"]
pub type OOVL4_R = crate::BitReader<bool>;
#[doc = "Field `OOVL4` writer - Output Override Value for PWML output of the channel 4"]
pub type OOVL4_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVL5` reader - Output Override Value for PWML output of the channel 5"]
pub type OOVL5_R = crate::BitReader<bool>;
#[doc = "Field `OOVL5` writer - Output Override Value for PWML output of the channel 5"]
pub type OOVL5_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVL6` reader - Output Override Value for PWML output of the channel 6"]
pub type OOVL6_R = crate::BitReader<bool>;
#[doc = "Field `OOVL6` writer - Output Override Value for PWML output of the channel 6"]
pub type OOVL6_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
#[doc = "Field `OOVL7` reader - Output Override Value for PWML output of the channel 7"]
pub type OOVL7_R = crate::BitReader<bool>;
#[doc = "Field `OOVL7` writer - Output Override Value for PWML output of the channel 7"]
pub type OOVL7_W<'a, const O: u8> = crate::BitWriter<'a, u32, OOV_SPEC, bool, O>;
impl R {
    #[doc = "Bit 0 - Output Override Value for PWMH output of the channel 0"]
    #[inline(always)]
    pub fn oovh0(&self) -> OOVH0_R {
        OOVH0_R::new((self.bits & 1) != 0)
    }
    #[doc = "Bit 1 - Output Override Value for PWMH output of the channel 1"]
    #[inline(always)]
    pub fn oovh1(&self) -> OOVH1_R {
        OOVH1_R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bit 2 - Output Override Value for PWMH output of the channel 2"]
    #[inline(always)]
    pub fn oovh2(&self) -> OOVH2_R {
        OOVH2_R::new(((self.bits >> 2) & 1) != 0)
    }
    #[doc = "Bit 3 - Output Override Value for PWMH output of the channel 3"]
    #[inline(always)]
    pub fn oovh3(&self) -> OOVH3_R {
        OOVH3_R::new(((self.bits >> 3) & 1) != 0)
    }
    #[doc = "Bit 4 - Output Override Value for PWMH output of the channel 4"]
    #[inline(always)]
    pub fn oovh4(&self) -> OOVH4_R {
        OOVH4_R::new(((self.bits >> 4) & 1) != 0)
    }
    #[doc = "Bit 5 - Output Override Value for PWMH output of the channel 5"]
    #[inline(always)]
    pub fn oovh5(&self) -> OOVH5_R {
        OOVH5_R::new(((self.bits >> 5) & 1) != 0)
    }
    #[doc = "Bit 6 - Output Override Value for PWMH output of the channel 6"]
    #[inline(always)]
    pub fn oovh6(&self) -> OOVH6_R {
        OOVH6_R::new(((self.bits >> 6) & 1) != 0)
    }
    #[doc = "Bit 7 - Output Override Value for PWMH output of the channel 7"]
    #[inline(always)]
    pub fn oovh7(&self) -> OOVH7_R {
        OOVH7_R::new(((self.bits >> 7) & 1) != 0)
    }
    #[doc = "Bit 16 - Output Override Value for PWML output of the channel 0"]
    #[inline(always)]
    pub fn oovl0(&self) -> OOVL0_R {
        OOVL0_R::new(((self.bits >> 16) & 1) != 0)
    }
    #[doc = "Bit 17 - Output Override Value for PWML output of the channel 1"]
    #[inline(always)]
    pub fn oovl1(&self) -> OOVL1_R {
        OOVL1_R::new(((self.bits >> 17) & 1) != 0)
    }
    #[doc = "Bit 18 - Output Override Value for PWML output of the channel 2"]
    #[inline(always)]
    pub fn oovl2(&self) -> OOVL2_R {
        OOVL2_R::new(((self.bits >> 18) & 1) != 0)
    }
    #[doc = "Bit 19 - Output Override Value for PWML output of the channel 3"]
    #[inline(always)]
    pub fn oovl3(&self) -> OOVL3_R {
        OOVL3_R::new(((self.bits >> 19) & 1) != 0)
    }
    #[doc = "Bit 20 - Output Override Value for PWML output of the channel 4"]
    #[inline(always)]
    pub fn oovl4(&self) -> OOVL4_R {
        OOVL4_R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bit 21 - Output Override Value for PWML output of the channel 5"]
    #[inline(always)]
    pub fn oovl5(&self) -> OOVL5_R {
        OOVL5_R::new(((self.bits >> 21) & 1) != 0)
    }
    #[doc = "Bit 22 - Output Override Value for PWML output of the channel 6"]
    #[inline(always)]
    pub fn oovl6(&self) -> OOVL6_R {
        OOVL6_R::new(((self.bits >> 22) & 1) != 0)
    }
    #[doc = "Bit 23 - Output Override Value for PWML output of the channel 7"]
    #[inline(always)]
    pub fn oovl7(&self) -> OOVL7_R {
        OOVL7_R::new(((self.bits >> 23) & 1) != 0)
    }
}
impl W {
    #[doc = "Bit 0 - Output Override Value for PWMH output of the channel 0"]
    #[inline(always)]
    pub fn oovh0(&mut self) -> OOVH0_W<0> {
        OOVH0_W::new(self)
    }
    #[doc = "Bit 1 - Output Override Value for PWMH output of the channel 1"]
    #[inline(always)]
    pub fn oovh1(&mut self) -> OOVH1_W<1> {
        OOVH1_W::new(self)
    }
    #[doc = "Bit 2 - Output Override Value for PWMH output of the channel 2"]
    #[inline(always)]
    pub fn oovh2(&mut self) -> OOVH2_W<2> {
        OOVH2_W::new(self)
    }
    #[doc = "Bit 3 - Output Override Value for PWMH output of the channel 3"]
    #[inline(always)]
    pub fn oovh3(&mut self) -> OOVH3_W<3> {
        OOVH3_W::new(self)
    }
    #[doc = "Bit 4 - Output Override Value for PWMH output of the channel 4"]
    #[inline(always)]
    pub fn oovh4(&mut self) -> OOVH4_W<4> {
        OOVH4_W::new(self)
    }
    #[doc = "Bit 5 - Output Override Value for PWMH output of the channel 5"]
    #[inline(always)]
    pub fn oovh5(&mut self) -> OOVH5_W<5> {
        OOVH5_W::new(self)
    }
    #[doc = "Bit 6 - Output Override Value for PWMH output of the channel 6"]
    #[inline(always)]
    pub fn oovh6(&mut self) -> OOVH6_W<6> {
        OOVH6_W::new(self)
    }
    #[doc = "Bit 7 - Output Override Value for PWMH output of the channel 7"]
    #[inline(always)]
    pub fn oovh7(&mut self) -> OOVH7_W<7> {
        OOVH7_W::new(self)
    }
    #[doc = "Bit 16 - Output Override Value for PWML output of the channel 0"]
    #[inline(always)]
    pub fn oovl0(&mut self) -> OOVL0_W<16> {
        OOVL0_W::new(self)
    }
    #[doc = "Bit 17 - Output Override Value for PWML output of the channel 1"]
    #[inline(always)]
    pub fn oovl1(&mut self) -> OOVL1_W<17> {
        OOVL1_W::new(self)
    }
    #[doc = "Bit 18 - Output Override Value for PWML output of the channel 2"]
    #[inline(always)]
    pub fn oovl2(&mut self) -> OOVL2_W<18> {
        OOVL2_W::new(self)
    }
    #[doc = "Bit 19 - Output Override Value for PWML output of the channel 3"]
    #[inline(always)]
    pub fn oovl3(&mut self) -> OOVL3_W<19> {
        OOVL3_W::new(self)
    }
    #[doc = "Bit 20 - Output Override Value for PWML output of the channel 4"]
    #[inline(always)]
    pub fn oovl4(&mut self) -> OOVL4_W<20> {
        OOVL4_W::new(self)
    }
    #[doc = "Bit 21 - Output Override Value for PWML output of the channel 5"]
    #[inline(always)]
    pub fn oovl5(&mut self) -> OOVL5_W<21> {
        OOVL5_W::new(self)
    }
    #[doc = "Bit 22 - Output Override Value for PWML output of the channel 6"]
    #[inline(always)]
    pub fn oovl6(&mut self) -> OOVL6_W<22> {
        OOVL6_W::new(self)
    }
    #[doc = "Bit 23 - Output Override Value for PWML output of the channel 7"]
    #[inline(always)]
    pub fn oovl7(&mut self) -> OOVL7_W<23> {
        OOVL7_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "PWM Output Override Value Register\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`reset`](crate::generic::Reg::reset), [`write`](crate::generic::Reg::write), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [oov](index.html) module"]
pub struct OOV_SPEC;
impl crate::RegisterSpec for OOV_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [oov::R](R) reader structure"]
impl crate::Readable for OOV_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [oov::W](W) writer structure"]
impl crate::Writable for OOV_SPEC {
    type Writer = W;
}
#[doc = "`reset()` method sets OOV to value 0"]
impl crate::Resettable for OOV_SPEC {
    #[inline(always)]
    fn reset_value() -> Self::Ux {
        0
    }
}
