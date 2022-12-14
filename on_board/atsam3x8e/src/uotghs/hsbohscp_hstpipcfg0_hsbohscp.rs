#[doc = "Register `HSTPIPCFG0_HSBOHSCP` reader"]
pub struct R(crate::R<HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC>);
impl core::ops::Deref for R {
    type Target = crate::R<HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<crate::R<HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC>> for R {
    #[inline(always)]
    fn from(reader: crate::R<HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC>) -> Self {
        R(reader)
    }
}
#[doc = "Register `HSTPIPCFG0_HSBOHSCP` writer"]
pub struct W(crate::W<HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC>);
impl core::ops::Deref for W {
    type Target = crate::W<HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC>;
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
impl From<crate::W<HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC>> for W {
    #[inline(always)]
    fn from(writer: crate::W<HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC>) -> Self {
        W(writer)
    }
}
#[doc = "Field `ALLOC` reader - Pipe Memory Allocate"]
pub type ALLOC_R = crate::BitReader<bool>;
#[doc = "Field `ALLOC` writer - Pipe Memory Allocate"]
pub type ALLOC_W<'a, const O: u8> =
    crate::BitWriter<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, bool, O>;
#[doc = "Field `PBK` reader - Pipe Banks"]
pub type PBK_R = crate::FieldReader<u8, PBK_A>;
#[doc = "Pipe Banks"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PBK_A {
    #[doc = "0: Single-bank pipe"]
    _1_BANK = 0,
    #[doc = "1: Double-bank pipe"]
    _2_BANK = 1,
    #[doc = "2: Triple-bank pipe"]
    _3_BANK = 2,
}
impl From<PBK_A> for u8 {
    #[inline(always)]
    fn from(variant: PBK_A) -> Self {
        variant as _
    }
}
impl PBK_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<PBK_A> {
        match self.bits {
            0 => Some(PBK_A::_1_BANK),
            1 => Some(PBK_A::_2_BANK),
            2 => Some(PBK_A::_3_BANK),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `_1_BANK`"]
    #[inline(always)]
    pub fn is_1_bank(&self) -> bool {
        *self == PBK_A::_1_BANK
    }
    #[doc = "Checks if the value of the field is `_2_BANK`"]
    #[inline(always)]
    pub fn is_2_bank(&self) -> bool {
        *self == PBK_A::_2_BANK
    }
    #[doc = "Checks if the value of the field is `_3_BANK`"]
    #[inline(always)]
    pub fn is_3_bank(&self) -> bool {
        *self == PBK_A::_3_BANK
    }
}
#[doc = "Field `PBK` writer - Pipe Banks"]
pub type PBK_W<'a, const O: u8> =
    crate::FieldWriter<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, u8, PBK_A, 2, O>;
impl<'a, const O: u8> PBK_W<'a, O> {
    #[doc = "Single-bank pipe"]
    #[inline(always)]
    pub fn _1_bank(self) -> &'a mut W {
        self.variant(PBK_A::_1_BANK)
    }
    #[doc = "Double-bank pipe"]
    #[inline(always)]
    pub fn _2_bank(self) -> &'a mut W {
        self.variant(PBK_A::_2_BANK)
    }
    #[doc = "Triple-bank pipe"]
    #[inline(always)]
    pub fn _3_bank(self) -> &'a mut W {
        self.variant(PBK_A::_3_BANK)
    }
}
#[doc = "Field `PSIZE` reader - Pipe Size"]
pub type PSIZE_R = crate::FieldReader<u8, PSIZE_A>;
#[doc = "Pipe Size"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PSIZE_A {
    #[doc = "0: 8 bytes"]
    _8_BYTE = 0,
    #[doc = "1: 16 bytes"]
    _16_BYTE = 1,
    #[doc = "2: 32 bytes"]
    _32_BYTE = 2,
    #[doc = "3: 64 bytes"]
    _64_BYTE = 3,
    #[doc = "4: 128 bytes"]
    _128_BYTE = 4,
    #[doc = "5: 256 bytes"]
    _256_BYTE = 5,
    #[doc = "6: 512 bytes"]
    _512_BYTE = 6,
    #[doc = "7: 1024 bytes"]
    _1024_BYTE = 7,
}
impl From<PSIZE_A> for u8 {
    #[inline(always)]
    fn from(variant: PSIZE_A) -> Self {
        variant as _
    }
}
impl PSIZE_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> PSIZE_A {
        match self.bits {
            0 => PSIZE_A::_8_BYTE,
            1 => PSIZE_A::_16_BYTE,
            2 => PSIZE_A::_32_BYTE,
            3 => PSIZE_A::_64_BYTE,
            4 => PSIZE_A::_128_BYTE,
            5 => PSIZE_A::_256_BYTE,
            6 => PSIZE_A::_512_BYTE,
            7 => PSIZE_A::_1024_BYTE,
            _ => unreachable!(),
        }
    }
    #[doc = "Checks if the value of the field is `_8_BYTE`"]
    #[inline(always)]
    pub fn is_8_byte(&self) -> bool {
        *self == PSIZE_A::_8_BYTE
    }
    #[doc = "Checks if the value of the field is `_16_BYTE`"]
    #[inline(always)]
    pub fn is_16_byte(&self) -> bool {
        *self == PSIZE_A::_16_BYTE
    }
    #[doc = "Checks if the value of the field is `_32_BYTE`"]
    #[inline(always)]
    pub fn is_32_byte(&self) -> bool {
        *self == PSIZE_A::_32_BYTE
    }
    #[doc = "Checks if the value of the field is `_64_BYTE`"]
    #[inline(always)]
    pub fn is_64_byte(&self) -> bool {
        *self == PSIZE_A::_64_BYTE
    }
    #[doc = "Checks if the value of the field is `_128_BYTE`"]
    #[inline(always)]
    pub fn is_128_byte(&self) -> bool {
        *self == PSIZE_A::_128_BYTE
    }
    #[doc = "Checks if the value of the field is `_256_BYTE`"]
    #[inline(always)]
    pub fn is_256_byte(&self) -> bool {
        *self == PSIZE_A::_256_BYTE
    }
    #[doc = "Checks if the value of the field is `_512_BYTE`"]
    #[inline(always)]
    pub fn is_512_byte(&self) -> bool {
        *self == PSIZE_A::_512_BYTE
    }
    #[doc = "Checks if the value of the field is `_1024_BYTE`"]
    #[inline(always)]
    pub fn is_1024_byte(&self) -> bool {
        *self == PSIZE_A::_1024_BYTE
    }
}
#[doc = "Field `PSIZE` writer - Pipe Size"]
pub type PSIZE_W<'a, const O: u8> =
    crate::FieldWriterSafe<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, u8, PSIZE_A, 3, O>;
impl<'a, const O: u8> PSIZE_W<'a, O> {
    #[doc = "8 bytes"]
    #[inline(always)]
    pub fn _8_byte(self) -> &'a mut W {
        self.variant(PSIZE_A::_8_BYTE)
    }
    #[doc = "16 bytes"]
    #[inline(always)]
    pub fn _16_byte(self) -> &'a mut W {
        self.variant(PSIZE_A::_16_BYTE)
    }
    #[doc = "32 bytes"]
    #[inline(always)]
    pub fn _32_byte(self) -> &'a mut W {
        self.variant(PSIZE_A::_32_BYTE)
    }
    #[doc = "64 bytes"]
    #[inline(always)]
    pub fn _64_byte(self) -> &'a mut W {
        self.variant(PSIZE_A::_64_BYTE)
    }
    #[doc = "128 bytes"]
    #[inline(always)]
    pub fn _128_byte(self) -> &'a mut W {
        self.variant(PSIZE_A::_128_BYTE)
    }
    #[doc = "256 bytes"]
    #[inline(always)]
    pub fn _256_byte(self) -> &'a mut W {
        self.variant(PSIZE_A::_256_BYTE)
    }
    #[doc = "512 bytes"]
    #[inline(always)]
    pub fn _512_byte(self) -> &'a mut W {
        self.variant(PSIZE_A::_512_BYTE)
    }
    #[doc = "1024 bytes"]
    #[inline(always)]
    pub fn _1024_byte(self) -> &'a mut W {
        self.variant(PSIZE_A::_1024_BYTE)
    }
}
#[doc = "Field `PTOKEN` reader - Pipe Token"]
pub type PTOKEN_R = crate::FieldReader<u8, PTOKEN_A>;
#[doc = "Pipe Token"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PTOKEN_A {
    #[doc = "0: SETUP"]
    SETUP = 0,
    #[doc = "1: IN"]
    IN = 1,
    #[doc = "2: OUT"]
    OUT = 2,
}
impl From<PTOKEN_A> for u8 {
    #[inline(always)]
    fn from(variant: PTOKEN_A) -> Self {
        variant as _
    }
}
impl PTOKEN_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<PTOKEN_A> {
        match self.bits {
            0 => Some(PTOKEN_A::SETUP),
            1 => Some(PTOKEN_A::IN),
            2 => Some(PTOKEN_A::OUT),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `SETUP`"]
    #[inline(always)]
    pub fn is_setup(&self) -> bool {
        *self == PTOKEN_A::SETUP
    }
    #[doc = "Checks if the value of the field is `IN`"]
    #[inline(always)]
    pub fn is_in(&self) -> bool {
        *self == PTOKEN_A::IN
    }
    #[doc = "Checks if the value of the field is `OUT`"]
    #[inline(always)]
    pub fn is_out(&self) -> bool {
        *self == PTOKEN_A::OUT
    }
}
#[doc = "Field `PTOKEN` writer - Pipe Token"]
pub type PTOKEN_W<'a, const O: u8> =
    crate::FieldWriter<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, u8, PTOKEN_A, 2, O>;
impl<'a, const O: u8> PTOKEN_W<'a, O> {
    #[doc = "SETUP"]
    #[inline(always)]
    pub fn setup(self) -> &'a mut W {
        self.variant(PTOKEN_A::SETUP)
    }
    #[doc = "IN"]
    #[inline(always)]
    pub fn in_(self) -> &'a mut W {
        self.variant(PTOKEN_A::IN)
    }
    #[doc = "OUT"]
    #[inline(always)]
    pub fn out(self) -> &'a mut W {
        self.variant(PTOKEN_A::OUT)
    }
}
#[doc = "Field `AUTOSW` reader - Automatic Switch"]
pub type AUTOSW_R = crate::BitReader<bool>;
#[doc = "Field `AUTOSW` writer - Automatic Switch"]
pub type AUTOSW_W<'a, const O: u8> =
    crate::BitWriter<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, bool, O>;
#[doc = "Field `PTYPE` reader - Pipe Type"]
pub type PTYPE_R = crate::FieldReader<u8, PTYPE_A>;
#[doc = "Pipe Type"]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum PTYPE_A {
    #[doc = "0: Control"]
    CTRL = 0,
    #[doc = "2: Bulk"]
    BLK = 2,
}
impl From<PTYPE_A> for u8 {
    #[inline(always)]
    fn from(variant: PTYPE_A) -> Self {
        variant as _
    }
}
impl PTYPE_R {
    #[doc = "Get enumerated values variant"]
    #[inline(always)]
    pub fn variant(&self) -> Option<PTYPE_A> {
        match self.bits {
            0 => Some(PTYPE_A::CTRL),
            2 => Some(PTYPE_A::BLK),
            _ => None,
        }
    }
    #[doc = "Checks if the value of the field is `CTRL`"]
    #[inline(always)]
    pub fn is_ctrl(&self) -> bool {
        *self == PTYPE_A::CTRL
    }
    #[doc = "Checks if the value of the field is `BLK`"]
    #[inline(always)]
    pub fn is_blk(&self) -> bool {
        *self == PTYPE_A::BLK
    }
}
#[doc = "Field `PTYPE` writer - Pipe Type"]
pub type PTYPE_W<'a, const O: u8> =
    crate::FieldWriter<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, u8, PTYPE_A, 2, O>;
impl<'a, const O: u8> PTYPE_W<'a, O> {
    #[doc = "Control"]
    #[inline(always)]
    pub fn ctrl(self) -> &'a mut W {
        self.variant(PTYPE_A::CTRL)
    }
    #[doc = "Bulk"]
    #[inline(always)]
    pub fn blk(self) -> &'a mut W {
        self.variant(PTYPE_A::BLK)
    }
}
#[doc = "Field `PEPNUM` reader - Pipe Endpoint Number"]
pub type PEPNUM_R = crate::FieldReader<u8, u8>;
#[doc = "Field `PEPNUM` writer - Pipe Endpoint Number"]
pub type PEPNUM_W<'a, const O: u8> =
    crate::FieldWriter<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, u8, u8, 4, O>;
#[doc = "Field `PINGEN` reader - Ping Enable"]
pub type PINGEN_R = crate::BitReader<bool>;
#[doc = "Field `PINGEN` writer - Ping Enable"]
pub type PINGEN_W<'a, const O: u8> =
    crate::BitWriter<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, bool, O>;
#[doc = "Field `BINTERVAL` reader - Binterval Parameter for the Bulk-Out/Ping Transaction"]
pub type BINTERVAL_R = crate::FieldReader<u8, u8>;
#[doc = "Field `BINTERVAL` writer - Binterval Parameter for the Bulk-Out/Ping Transaction"]
pub type BINTERVAL_W<'a, const O: u8> =
    crate::FieldWriter<'a, u32, HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC, u8, u8, 8, O>;
impl R {
    #[doc = "Bit 1 - Pipe Memory Allocate"]
    #[inline(always)]
    pub fn alloc(&self) -> ALLOC_R {
        ALLOC_R::new(((self.bits >> 1) & 1) != 0)
    }
    #[doc = "Bits 2:3 - Pipe Banks"]
    #[inline(always)]
    pub fn pbk(&self) -> PBK_R {
        PBK_R::new(((self.bits >> 2) & 3) as u8)
    }
    #[doc = "Bits 4:6 - Pipe Size"]
    #[inline(always)]
    pub fn psize(&self) -> PSIZE_R {
        PSIZE_R::new(((self.bits >> 4) & 7) as u8)
    }
    #[doc = "Bits 8:9 - Pipe Token"]
    #[inline(always)]
    pub fn ptoken(&self) -> PTOKEN_R {
        PTOKEN_R::new(((self.bits >> 8) & 3) as u8)
    }
    #[doc = "Bit 10 - Automatic Switch"]
    #[inline(always)]
    pub fn autosw(&self) -> AUTOSW_R {
        AUTOSW_R::new(((self.bits >> 10) & 1) != 0)
    }
    #[doc = "Bits 12:13 - Pipe Type"]
    #[inline(always)]
    pub fn ptype(&self) -> PTYPE_R {
        PTYPE_R::new(((self.bits >> 12) & 3) as u8)
    }
    #[doc = "Bits 16:19 - Pipe Endpoint Number"]
    #[inline(always)]
    pub fn pepnum(&self) -> PEPNUM_R {
        PEPNUM_R::new(((self.bits >> 16) & 0x0f) as u8)
    }
    #[doc = "Bit 20 - Ping Enable"]
    #[inline(always)]
    pub fn pingen(&self) -> PINGEN_R {
        PINGEN_R::new(((self.bits >> 20) & 1) != 0)
    }
    #[doc = "Bits 24:31 - Binterval Parameter for the Bulk-Out/Ping Transaction"]
    #[inline(always)]
    pub fn binterval(&self) -> BINTERVAL_R {
        BINTERVAL_R::new(((self.bits >> 24) & 0xff) as u8)
    }
}
impl W {
    #[doc = "Bit 1 - Pipe Memory Allocate"]
    #[inline(always)]
    #[must_use]
    pub fn alloc(&mut self) -> ALLOC_W<1> {
        ALLOC_W::new(self)
    }
    #[doc = "Bits 2:3 - Pipe Banks"]
    #[inline(always)]
    #[must_use]
    pub fn pbk(&mut self) -> PBK_W<2> {
        PBK_W::new(self)
    }
    #[doc = "Bits 4:6 - Pipe Size"]
    #[inline(always)]
    #[must_use]
    pub fn psize(&mut self) -> PSIZE_W<4> {
        PSIZE_W::new(self)
    }
    #[doc = "Bits 8:9 - Pipe Token"]
    #[inline(always)]
    #[must_use]
    pub fn ptoken(&mut self) -> PTOKEN_W<8> {
        PTOKEN_W::new(self)
    }
    #[doc = "Bit 10 - Automatic Switch"]
    #[inline(always)]
    #[must_use]
    pub fn autosw(&mut self) -> AUTOSW_W<10> {
        AUTOSW_W::new(self)
    }
    #[doc = "Bits 12:13 - Pipe Type"]
    #[inline(always)]
    #[must_use]
    pub fn ptype(&mut self) -> PTYPE_W<12> {
        PTYPE_W::new(self)
    }
    #[doc = "Bits 16:19 - Pipe Endpoint Number"]
    #[inline(always)]
    #[must_use]
    pub fn pepnum(&mut self) -> PEPNUM_W<16> {
        PEPNUM_W::new(self)
    }
    #[doc = "Bit 20 - Ping Enable"]
    #[inline(always)]
    #[must_use]
    pub fn pingen(&mut self) -> PINGEN_W<20> {
        PINGEN_W::new(self)
    }
    #[doc = "Bits 24:31 - Binterval Parameter for the Bulk-Out/Ping Transaction"]
    #[inline(always)]
    #[must_use]
    pub fn binterval(&mut self) -> BINTERVAL_W<24> {
        BINTERVAL_W::new(self)
    }
    #[doc = "Writes raw bits to the register."]
    #[inline(always)]
    pub unsafe fn bits(&mut self, bits: u32) -> &mut Self {
        self.0.bits(bits);
        self
    }
}
#[doc = "Host Pipe Configuration Register (n = 0)\n\nThis register you can [`read`](crate::generic::Reg::read), [`write_with_zero`](crate::generic::Reg::write_with_zero), [`modify`](crate::generic::Reg::modify). See [API](https://docs.rs/svd2rust/#read--modify--write-api).\n\nFor information about available fields see [hsbohscp_hstpipcfg0_hsbohscp](index.html) module"]
pub struct HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC;
impl crate::RegisterSpec for HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC {
    type Ux = u32;
}
#[doc = "`read()` method returns [hsbohscp_hstpipcfg0_hsbohscp::R](R) reader structure"]
impl crate::Readable for HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC {
    type Reader = R;
}
#[doc = "`write(|w| ..)` method takes [hsbohscp_hstpipcfg0_hsbohscp::W](W) writer structure"]
impl crate::Writable for HSBOHSCP_HSTPIPCFG0_HSBOHSCP_SPEC {
    type Writer = W;
    const ZERO_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
    const ONE_TO_MODIFY_FIELDS_BITMAP: Self::Ux = 0;
}
