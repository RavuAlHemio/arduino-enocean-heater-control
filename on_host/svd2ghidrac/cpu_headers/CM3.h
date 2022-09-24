#pragma pack(push,1)

#ifdef GHIDRA_STDINT
typedef sbyte int8_t;
typedef sword int16_t;
typedef sdword int32_t;
typedef byte uint8_t;
typedef word uint16_t;
typedef dword uint32_t;
#endif

typedef void ExceptionHandler(void);

/** Auxiliary Control Register */
typedef struct {
  uint32_t DISMCYCINT : 1;
  uint32_t DISDEFWBUF : 1;
  uint32_t DISFOLD : 1;
  uint32_t reserved_tail : 29;
} SCB_ACTLR;

/** CPUID Base Register */
typedef struct {
  uint32_t Revision : 4;
  uint32_t PartNo : 12;
  uint32_t Constant : 4;
  uint32_t Variant : 4;
  uint32_t Implementer : 8;
} SCB_CPUID;

/** Interrupt Control and State Register */
typedef struct {
  uint32_t VECTACTIVE : 9;
  uint32_t reserved0 : 2;
  uint32_t RETTOBASE : 1;
  uint32_t VECTPENDING : 10;
  uint32_t ISRPENDING : 1;
  uint32_t reserved_for_debug : 1;
  uint32_t reserved1 : 1;
  uint32_t PENDSTCLR : 1;
  uint32_t PENDSTSET : 1;
  uint32_t PENDSVCLR : 1;
  uint32_t PENDSVSET : 1;
  uint32_t reserved_tail : 3;
} SCB_ICSR;

/** Vector Table Offset Register */
typedef struct {
  uint32_t reserved0 : 7;
  uint32_t TBLOFF : 23;
  uint32_t reserved_tail : 2;
} SCB_VTOR;

/** Application Interrupt and Reset Control Register */
typedef struct {
  uint32_t VECTRESET : 1;
  uint32_t VECTCLRACTIVE : 1;
  uint32_t SYSRESETREQ : 1;
  uint32_t reserved0 : 5;
  uint32_t PRIGROUP : 3;
  uint32_t reserved1 : 4;
  uint32_t ENDIANESS : 1;
  uint32_t VECTKEY : 16;
} SCB_AIRCR;

/** System Control Register */
typedef struct {
  uint32_t reserved0 : 1;
  uint32_t SLEEONEXIT : 1;
  uint32_t SLEEPDEEP : 1;
  uint32_t reserved1 : 1;
  uint32_t SEVONPEND : 1;
  uint32_t reserved_tail : 27;
} SCB_SCR;

/** Configuration and Control Register */
typedef struct {
  uint32_t NONBASETHRDENA : 1;
  uint32_t USERSETMPEND : 1;
  uint32_t reserved0 : 1;
  uint32_t UNALIGN_TRP : 1;
  uint32_t DIV_0_TRP : 1;
  uint32_t reserved1 : 3;
  uint32_t BFHFNMIGN : 1;
  uint32_t STKALIGN : 1;
  uint32_t reserved_tail : 22;
} SCB_CCR;

/** System Handler Priority Register */
typedef struct {
  uint32_t PRI_0 : 8;
  uint32_t PRI_1 : 8;
  uint32_t PRI_2 : 8;
  uint32_t PRI_3 : 8;
} SCB_SHPR;

/** System Handler Control and State Register */
typedef struct {
  uint32_t MEMFAULTACT : 1;
  uint32_t BUSFAULTACT : 1;
  uint32_t reserved0 : 1;
  uint32_t USGFAULTACT : 1;
  uint32_t reserved1 : 3;
  uint32_t SVCALLAVCT : 1;
  uint32_t MONITORACT : 1;
  uint32_t reserved2 : 1;
  uint32_t PENDSVACT : 1;
  uint32_t SYSTICKACT : 1;
  uint32_t USGFAULTPENDED : 1;
  uint32_t MEMFAULTPENDED : 1;
  uint32_t BUSFAULTPENDED : 1;
  uint32_t SVCALLPENDED : 1;
  uint32_t MEMFAULTENA : 1;
  uint32_t BUSFAULTENA : 1;
  uint32_t USGFAULTENA : 1;
  uint32_t reserved_tail : 13;
} SCB_SHCRS;

/** System Handler Control and State Register */
typedef struct {
  uint32_t MMFSR_IACCVIOL : 1;
  uint32_t MMFSR_DACCVIOL : 1;
  uint32_t reserved0 : 1;
  uint32_t MMFSR_MUNSTKERR : 1;
  uint32_t MMFSR_MSTKERR : 1;
  uint32_t reserved1 : 2;
  uint32_t MMFSR_MMARVALID : 1;

  uint32_t BFSR_IBUSERR : 1;
  uint32_t BFSR_PRECISERR : 1;
  uint32_t BFSR_IMPRECISERR : 1;
  uint32_t BFSR_UNSTKERR : 1;
  uint32_t BFSR_STKERR : 1;
  uint32_t reserved2 : 2;
  uint32_t BFSR_BFRVALID : 1;

  uint32_t UFSR_UNDEFINSTR : 1;
  uint32_t UFSR_INVSTATE : 1;
  uint32_t UFSR_INVPC : 1;
  uint32_t UFSR_NOCP : 1;
  uint32_t reserved3 : 4;
  uint32_t UFSR_UNALIGNED : 1;
  uint32_t UFSR_DIVBYZERO : 1;
  uint32_t reserved_tail : 6;
} SCB_CFSR;

/** Hard Fault Status Register */
typedef struct {
  uint32_t reserved0 : 1;
  uint32_t VECTTBL : 1;
  uint32_t reserved1 : 28;
  uint32_t FORCED : 1;
  uint32_t DEBUGEVT : 1;
} SCB_HFSR;

/** Memory Management Fault Address Register */
typedef struct {
  uint32_t ADDRESS : 32;
} SCB_MMFAR;

/** Bus Fault Address Register */
typedef struct {
  uint32_t ADDRESS : 32;
} SCB_BFAR;

/** System Control Block, first part */
typedef struct {
  SCB_ACTLR ACTLR;
} SCBpart0;

/** System Control Block, second part */
typedef struct {
  SCB_CPUID CPUID;
  SCB_ICSR ICSR;
  SCB_VTOR VTOR;
  SCB_AIRCR AIRCR;
  SCB_SCR SCR;
  SCB_CCR CCR;
  SCB_SHPR SHPR[3];
  SCB_SHCRS SHCRS;
  SCB_CFSR CFSR;
  SCB_HFSR HFSR;
  uint32_t reserved1;
  SCB_MMFAR MMFAR;
  SCB_BFAR BFAR;
} SCBpart1;

/** SysTick Control and Status Register */
typedef struct {
  uint32_t ENABLE : 1;
  uint32_t TICKINT : 1;
  uint32_t CLKSOURCE : 1;
  uint32_t reserved0 : 13;
  uint32_t COUNTFLAG : 1;
  uint32_t reserved_tail : 15;
} SysTick_CTRL;

/** SysTick Reload Value Register */
typedef struct {
  uint32_t RELOAD : 24;
  uint32_t reserved_tail : 8;
} SysTick_LOAD;

/** SysTick Current Value Register */
typedef struct {
  uint32_t CURRENT : 24;
  uint32_t reserved_tail : 8;
} SysTick_VAL;

/** SysTick Calibration Value Register */
typedef struct {
  uint32_t TENMS : 24;
  uint32_t reserved0 : 6;
  uint32_t SKEW : 1;
  uint32_t NOREF : 1;
} SysTick_CALIB;

/** System Timer (SysTick) */
typedef struct {
  SysTick_CTRL CTRL;
  SysTick_LOAD LOAD;
  SysTick_VAL VAL;
  SysTick_CALIB CALIB;
} SysTick;

/** Interrupt Set-Enable Register */
typedef struct {
  /** Enable interrupt 0 */
  uint32_t SETENA0 : 1;
  /** Enable interrupt 1 */
  uint32_t SETENA1 : 1;
  /** Enable interrupt 2 */
  uint32_t SETENA2 : 1;
  /** Enable interrupt 3 */
  uint32_t SETENA3 : 1;
  /** Enable interrupt 4 */
  uint32_t SETENA4 : 1;
  /** Enable interrupt 5 */
  uint32_t SETENA5 : 1;
  /** Enable interrupt 6 */
  uint32_t SETENA6 : 1;
  /** Enable interrupt 7 */
  uint32_t SETENA7 : 1;
  /** Enable interrupt 8 */
  uint32_t SETENA8 : 1;
  /** Enable interrupt 9 */
  uint32_t SETENA9 : 1;
  /** Enable interrupt 10 */
  uint32_t SETENA10 : 1;
  /** Enable interrupt 11 */
  uint32_t SETENA11 : 1;
  /** Enable interrupt 12 */
  uint32_t SETENA12 : 1;
  /** Enable interrupt 13 */
  uint32_t SETENA13 : 1;
  /** Enable interrupt 14 */
  uint32_t SETENA14 : 1;
  /** Enable interrupt 15 */
  uint32_t SETENA15 : 1;
  /** Enable interrupt 16 */
  uint32_t SETENA16 : 1;
  /** Enable interrupt 17 */
  uint32_t SETENA17 : 1;
  /** Enable interrupt 18 */
  uint32_t SETENA18 : 1;
  /** Enable interrupt 19 */
  uint32_t SETENA19 : 1;
  /** Enable interrupt 20 */
  uint32_t SETENA20 : 1;
  /** Enable interrupt 21 */
  uint32_t SETENA21 : 1;
  /** Enable interrupt 22 */
  uint32_t SETENA22 : 1;
  /** Enable interrupt 23 */
  uint32_t SETENA23 : 1;
  /** Enable interrupt 24 */
  uint32_t SETENA24 : 1;
  /** Enable interrupt 25 */
  uint32_t SETENA25 : 1;
  /** Enable interrupt 26 */
  uint32_t SETENA26 : 1;
  /** Enable interrupt 27 */
  uint32_t SETENA27 : 1;
  /** Enable interrupt 28 */
  uint32_t SETENA28 : 1;
  /** Enable interrupt 29 */
  uint32_t SETENA29 : 1;
  /** Enable interrupt 30 */
  uint32_t SETENA30 : 1;
  /** Enable interrupt 31 */
  uint32_t SETENA31 : 1;
} NVIC_ISER;

/** Interrupt Clear-Enable Register */
typedef struct {
  /** Disable interrupt 0 */
  uint32_t CLRENA0 : 1;
  /** Disable interrupt 1 */
  uint32_t CLRENA1 : 1;
  /** Disable interrupt 2 */
  uint32_t CLRENA2 : 1;
  /** Disable interrupt 3 */
  uint32_t CLRENA3 : 1;
  /** Disable interrupt 4 */
  uint32_t CLRENA4 : 1;
  /** Disable interrupt 5 */
  uint32_t CLRENA5 : 1;
  /** Disable interrupt 6 */
  uint32_t CLRENA6 : 1;
  /** Disable interrupt 7 */
  uint32_t CLRENA7 : 1;
  /** Disable interrupt 8 */
  uint32_t CLRENA8 : 1;
  /** Disable interrupt 9 */
  uint32_t CLRENA9 : 1;
  /** Disable interrupt 10 */
  uint32_t CLRENA10 : 1;
  /** Disable interrupt 11 */
  uint32_t CLRENA11 : 1;
  /** Disable interrupt 12 */
  uint32_t CLRENA12 : 1;
  /** Disable interrupt 13 */
  uint32_t CLRENA13 : 1;
  /** Disable interrupt 14 */
  uint32_t CLRENA14 : 1;
  /** Disable interrupt 15 */
  uint32_t CLRENA15 : 1;
  /** Disable interrupt 16 */
  uint32_t CLRENA16 : 1;
  /** Disable interrupt 17 */
  uint32_t CLRENA17 : 1;
  /** Disable interrupt 18 */
  uint32_t CLRENA18 : 1;
  /** Disable interrupt 19 */
  uint32_t CLRENA19 : 1;
  /** Disable interrupt 20 */
  uint32_t CLRENA20 : 1;
  /** Disable interrupt 21 */
  uint32_t CLRENA21 : 1;
  /** Disable interrupt 22 */
  uint32_t CLRENA22 : 1;
  /** Disable interrupt 23 */
  uint32_t CLRENA23 : 1;
  /** Disable interrupt 24 */
  uint32_t CLRENA24 : 1;
  /** Disable interrupt 25 */
  uint32_t CLRENA25 : 1;
  /** Disable interrupt 26 */
  uint32_t CLRENA26 : 1;
  /** Disable interrupt 27 */
  uint32_t CLRENA27 : 1;
  /** Disable interrupt 28 */
  uint32_t CLRENA28 : 1;
  /** Disable interrupt 29 */
  uint32_t CLRENA29 : 1;
  /** Disable interrupt 30 */
  uint32_t CLRENA30 : 1;
  /** Disable interrupt 31 */
  uint32_t CLRENA31 : 1;
} NVIC_ICER;

/** Interrupt Set-Pending Register */
typedef struct {
  /** Set interrupt 0 pending */
  uint32_t SETPEND0 : 1;
  /** Set interrupt 1 pending */
  uint32_t SETPEND1 : 1;
  /** Set interrupt 2 pending */
  uint32_t SETPEND2 : 1;
  /** Set interrupt 3 pending */
  uint32_t SETPEND3 : 1;
  /** Set interrupt 4 pending */
  uint32_t SETPEND4 : 1;
  /** Set interrupt 5 pending */
  uint32_t SETPEND5 : 1;
  /** Set interrupt 6 pending */
  uint32_t SETPEND6 : 1;
  /** Set interrupt 7 pending */
  uint32_t SETPEND7 : 1;
  /** Set interrupt 8 pending */
  uint32_t SETPEND8 : 1;
  /** Set interrupt 9 pending */
  uint32_t SETPEND9 : 1;
  /** Set interrupt 10 pending */
  uint32_t SETPEND10 : 1;
  /** Set interrupt 11 pending */
  uint32_t SETPEND11 : 1;
  /** Set interrupt 12 pending */
  uint32_t SETPEND12 : 1;
  /** Set interrupt 13 pending */
  uint32_t SETPEND13 : 1;
  /** Set interrupt 14 pending */
  uint32_t SETPEND14 : 1;
  /** Set interrupt 15 pending */
  uint32_t SETPEND15 : 1;
  /** Set interrupt 16 pending */
  uint32_t SETPEND16 : 1;
  /** Set interrupt 17 pending */
  uint32_t SETPEND17 : 1;
  /** Set interrupt 18 pending */
  uint32_t SETPEND18 : 1;
  /** Set interrupt 19 pending */
  uint32_t SETPEND19 : 1;
  /** Set interrupt 20 pending */
  uint32_t SETPEND20 : 1;
  /** Set interrupt 21 pending */
  uint32_t SETPEND21 : 1;
  /** Set interrupt 22 pending */
  uint32_t SETPEND22 : 1;
  /** Set interrupt 23 pending */
  uint32_t SETPEND23 : 1;
  /** Set interrupt 24 pending */
  uint32_t SETPEND24 : 1;
  /** Set interrupt 25 pending */
  uint32_t SETPEND25 : 1;
  /** Set interrupt 26 pending */
  uint32_t SETPEND26 : 1;
  /** Set interrupt 27 pending */
  uint32_t SETPEND27 : 1;
  /** Set interrupt 28 pending */
  uint32_t SETPEND28 : 1;
  /** Set interrupt 29 pending */
  uint32_t SETPEND29 : 1;
  /** Set interrupt 30 pending */
  uint32_t SETPEND30 : 1;
  /** Set interrupt 31 pending */
  uint32_t SETPEND31 : 1;
} NVIC_ISPR;

/** Interrupt Clear-Pending Register */
typedef struct {
  /** Clear interrupt 0 pending */
  uint32_t CLRPEND0 : 1;
  /** Clear interrupt 1 pending */
  uint32_t CLRPEND1 : 1;
  /** Clear interrupt 2 pending */
  uint32_t CLRPEND2 : 1;
  /** Clear interrupt 3 pending */
  uint32_t CLRPEND3 : 1;
  /** Clear interrupt 4 pending */
  uint32_t CLRPEND4 : 1;
  /** Clear interrupt 5 pending */
  uint32_t CLRPEND5 : 1;
  /** Clear interrupt 6 pending */
  uint32_t CLRPEND6 : 1;
  /** Clear interrupt 7 pending */
  uint32_t CLRPEND7 : 1;
  /** Clear interrupt 8 pending */
  uint32_t CLRPEND8 : 1;
  /** Clear interrupt 9 pending */
  uint32_t CLRPEND9 : 1;
  /** Clear interrupt 10 pending */
  uint32_t CLRPEND10 : 1;
  /** Clear interrupt 11 pending */
  uint32_t CLRPEND11 : 1;
  /** Clear interrupt 12 pending */
  uint32_t CLRPEND12 : 1;
  /** Clear interrupt 13 pending */
  uint32_t CLRPEND13 : 1;
  /** Clear interrupt 14 pending */
  uint32_t CLRPEND14 : 1;
  /** Clear interrupt 15 pending */
  uint32_t CLRPEND15 : 1;
  /** Clear interrupt 16 pending */
  uint32_t CLRPEND16 : 1;
  /** Clear interrupt 17 pending */
  uint32_t CLRPEND17 : 1;
  /** Clear interrupt 18 pending */
  uint32_t CLRPEND18 : 1;
  /** Clear interrupt 19 pending */
  uint32_t CLRPEND19 : 1;
  /** Clear interrupt 20 pending */
  uint32_t CLRPEND20 : 1;
  /** Clear interrupt 21 pending */
  uint32_t CLRPEND21 : 1;
  /** Clear interrupt 22 pending */
  uint32_t CLRPEND22 : 1;
  /** Clear interrupt 23 pending */
  uint32_t CLRPEND23 : 1;
  /** Clear interrupt 24 pending */
  uint32_t CLRPEND24 : 1;
  /** Clear interrupt 25 pending */
  uint32_t CLRPEND25 : 1;
  /** Clear interrupt 26 pending */
  uint32_t CLRPEND26 : 1;
  /** Clear interrupt 27 pending */
  uint32_t CLRPEND27 : 1;
  /** Clear interrupt 28 pending */
  uint32_t CLRPEND28 : 1;
  /** Clear interrupt 29 pending */
  uint32_t CLRPEND29 : 1;
  /** Clear interrupt 30 pending */
  uint32_t CLRPEND30 : 1;
  /** Clear interrupt 31 pending */
  uint32_t CLRPEND31 : 1;
} NVIC_ICPR;

/** Interrupt Active Register */
typedef struct {
  /** Is interrupt 0 active? */
  uint32_t ACTIVE0 : 1;
  /** Is interrupt 1 active? */
  uint32_t ACTIVE1 : 1;
  /** Is interrupt 2 active? */
  uint32_t ACTIVE2 : 1;
  /** Is interrupt 3 active? */
  uint32_t ACTIVE3 : 1;
  /** Is interrupt 4 active? */
  uint32_t ACTIVE4 : 1;
  /** Is interrupt 5 active? */
  uint32_t ACTIVE5 : 1;
  /** Is interrupt 6 active? */
  uint32_t ACTIVE6 : 1;
  /** Is interrupt 7 active? */
  uint32_t ACTIVE7 : 1;
  /** Is interrupt 8 active? */
  uint32_t ACTIVE8 : 1;
  /** Is interrupt 9 active? */
  uint32_t ACTIVE9 : 1;
  /** Is interrupt 10 active? */
  uint32_t ACTIVE10 : 1;
  /** Is interrupt 11 active? */
  uint32_t ACTIVE11 : 1;
  /** Is interrupt 12 active? */
  uint32_t ACTIVE12 : 1;
  /** Is interrupt 13 active? */
  uint32_t ACTIVE13 : 1;
  /** Is interrupt 14 active? */
  uint32_t ACTIVE14 : 1;
  /** Is interrupt 15 active? */
  uint32_t ACTIVE15 : 1;
  /** Is interrupt 16 active? */
  uint32_t ACTIVE16 : 1;
  /** Is interrupt 17 active? */
  uint32_t ACTIVE17 : 1;
  /** Is interrupt 18 active? */
  uint32_t ACTIVE18 : 1;
  /** Is interrupt 19 active? */
  uint32_t ACTIVE19 : 1;
  /** Is interrupt 20 active? */
  uint32_t ACTIVE20 : 1;
  /** Is interrupt 21 active? */
  uint32_t ACTIVE21 : 1;
  /** Is interrupt 22 active? */
  uint32_t ACTIVE22 : 1;
  /** Is interrupt 23 active? */
  uint32_t ACTIVE23 : 1;
  /** Is interrupt 24 active? */
  uint32_t ACTIVE24 : 1;
  /** Is interrupt 25 active? */
  uint32_t ACTIVE25 : 1;
  /** Is interrupt 26 active? */
  uint32_t ACTIVE26 : 1;
  /** Is interrupt 27 active? */
  uint32_t ACTIVE27 : 1;
  /** Is interrupt 28 active? */
  uint32_t ACTIVE28 : 1;
  /** Is interrupt 29 active? */
  uint32_t ACTIVE29 : 1;
  /** Is interrupt 30 active? */
  uint32_t ACTIVE30 : 1;
  /** Is interrupt 31 active? */
  uint32_t ACTIVE31 : 1;
} NVIC_IABR;

/** Interrupt Priority Register */
typedef struct {
  /** Interrupt 0 priority */
  uint32_t IP0 : 4;
  /** Interrupt 1 priority */
  uint32_t IP1 : 4;
  /** Interrupt 2 priority */
  uint32_t IP2 : 4;
  /** Interrupt 3 priority */
  uint32_t IP3 : 4;
  /** Interrupt 4 priority */
  uint32_t IP4 : 4;
  /** Interrupt 5 priority */
  uint32_t IP5 : 4;
  /** Interrupt 6 priority */
  uint32_t IP6 : 4;
  /** Interrupt 7 priority */
  uint32_t IP7 : 4;
} IPR;

/** Interrupt Priority Register */
typedef struct {
  /** Interrupt 0 priority */
  uint32_t IP0 : 4;
  /** Interrupt 1 priority */
  uint32_t IP1 : 4;
  /** Interrupt 2 priority */
  uint32_t IP2 : 4;
  /** Interrupt 3 priority */
  uint32_t IP3 : 4;
  /** Interrupt 4 priority */
  uint32_t IP4 : 4;
  /** Interrupt 5 priority */
  uint32_t IP5 : 4;
  /** Interrupt 6 priority */
  uint32_t IP6 : 4;
  /** Interrupt 7 priority */
  uint32_t IP7 : 4;
} NVIC_IPR;

/** Software Trigger Interrupt Register */
typedef struct {
  /** ID of interrupt to trigger */
  uint32_t INTID : 9;
  uint32_t reserved_tail : 23;
} NVIC_STIR;

/** Nested Vectored Interrupt Controller, first part */
typedef struct {
  NVIC_ISER ISER[2];
  uint32_t reserved0[30];
  NVIC_ICER ICER[2];
  uint32_t reserved1[30];
  NVIC_ISPR ISPR[2];
  uint32_t reserved2[30];
  NVIC_ICPR ICPR[2];
  uint32_t reserved3[30];
  NVIC_IABR IABR[2];
  uint32_t reserved3[62];
  NVIC_IPR IPR[8];
} NVICpart0;

/** Nested Vectored Interrupt Controller, second part */
typedef struct {
  NVIC_STIR STIR;
} NVICpart1;

/** MPU Type Register */
typedef struct {
  uint32_t SEPARATE : 1;
  uint32_t reserved0 : 7;
  uint32_t DREGION : 8;
  uint32_t IREGION : 8;
  uint32_t reserved_tail : 8;
} MPU_TYPE;

/** MPU Control Register */
typedef struct {
  uint32_t ENABLE : 1;
  uint32_t HFNMIENA : 1;
  uint32_t PRIVDEFENA : 1;
  uint32_t reserved_tail : 29;
} MPU_CTRL;

/** MPU Region Number Register */
typedef struct {
  uint32_t REGION : 8;
  uint32_t reserved_tail : 24;
} MPU_RNR;

/** MPU Region Base Address Register */
typedef struct {
  uint32_t REGION : 4;
  uint32_t VALID : 1;
  uint32_t reserved0 : 3;
  uint32_t ADDR : 24;
} MPU_RBAR;

/** MPU Region Attribute and Size Register */
typedef struct {
  uint32_t ENABLE : 1;
  uint32_t SIZE : 5;
  uint32_t reserved0 : 2;
  uint32_t SRD : 8;
  uint32_t B : 1;
  uint32_t C : 1;
  uint32_t S : 1;
  uint32_t TEX : 3;
  uint32_t reserved1 : 2;
  uint32_t AP : 3;
  uint32_t reserved2 : 1;
  uint32_t XN : 1;
  uint32_t reserved_tail : 3;
} MPU_RASR;

/** Memory Protection Unit */
typedef struct {
  MPU_TYPE TYPE;
  MPU_CTRL CTRL;
  MPU_RNR RNR;
  MPU_RBAR RBAR;
  MPU_RASR RASR;
  MPU_RBAR RBAR_A1;
  MPU_RASR RASR_A1;
  MPU_RBAR RBAR_A2;
  MPU_RASR RASR_A2;
  MPU_RBAR RBAR_A3;
  MPU_RASR RASR_A3;
} MPU;

#pragma pack(pop)
