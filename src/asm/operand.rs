use iced_x86::code_asm::{AsmMemoryOperand, AsmRegister16, AsmRegister32, AsmRegister64, AsmRegister8, AsmRegisterBnd, AsmRegisterCr, AsmRegisterDr, AsmRegisterK, AsmRegisterMm, AsmRegisterSegment, AsmRegisterSt, AsmRegisterTmm, AsmRegisterTr, AsmRegisterXmm, AsmRegisterYmm, AsmRegisterZmm};

pub enum Operand
{
    Reg8(AsmRegister8),
    Reg16(AsmRegister16),
    Reg32(AsmRegister32),
    Reg64(AsmRegister64),
    RegXmm(AsmRegisterXmm),
    RegYmm(AsmRegisterYmm),
    RegZmm(AsmRegisterZmm),
    RegK(AsmRegisterK),
    RegBnd(AsmRegisterBnd),
    RegCr(AsmRegisterCr),
    RegDr(AsmRegisterDr),
    RegSt(AsmRegisterSt),
    RegMm(AsmRegisterMm),
    RegTr(AsmRegisterTr),
    RegTmm(AsmRegisterTmm),
    RegSegment(AsmRegisterSegment),

    Immediate(i32),

    Memory(AsmMemoryOperand),
}

impl Operand
{
    
}

impl TryInto<AsmRegister8> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegister8, Self::Error>
    {
        match self
        {
            Operand::Reg8(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegister16> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegister16, Self::Error>
    {
        match self
        {
            Operand::Reg16(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegister32> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegister32, Self::Error>
    {
        match self
        {
            Operand::Reg32(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegister64> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegister64, Self::Error>
    {
        match self
        {
            Operand::Reg64(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterXmm> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterXmm, Self::Error>
    {
        match self
        {
            Operand::RegXmm(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterYmm> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterYmm, Self::Error>
    {
        match self
        {
            Operand::RegYmm(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterZmm> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterZmm, Self::Error>
    {
        match self
        {
            Operand::RegZmm(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterK> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterK, Self::Error>
    {
        match self
        {
            Operand::RegK(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterBnd> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterBnd, Self::Error>
    {
        match self
        {
            Operand::RegBnd(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterCr> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterCr, Self::Error>
    {
        match self
        {
            Operand::RegCr(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterDr> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterDr, Self::Error>
    {
        match self
        {
            Operand::RegDr(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterSt> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterSt, Self::Error>
    {
        match self
        {
            Operand::RegSt(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterMm> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterMm, Self::Error>
    {
        match self
        {
            Operand::RegMm(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterTr> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterTr, Self::Error>
    {
        match self
        {
            Operand::RegTr(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterTmm> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterTmm, Self::Error>
    {
        match self
        {
            Operand::RegTmm(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmRegisterSegment> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmRegisterSegment, Self::Error>
    {
        match self
        {
            Operand::RegSegment(reg) => Ok(reg),
            _ => Err(()),
        }
    }
}

impl TryInto<AsmMemoryOperand> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<AsmMemoryOperand, Self::Error>
    {
        match self
        {
            Operand::Memory(mem) => Ok(mem),
            _ => Err(()),
        }
    }
}

impl TryInto<i32> for Operand
{
    type Error = ();
    fn try_into(self) -> Result<i32, Self::Error>
    {
        match self
        {
            Operand::Immediate(imm) => Ok(imm),
            _ => Err(()),
        }
    }
}
