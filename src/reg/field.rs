use super::*;
use core::ptr::{read_volatile, write_volatile};

/// Register field binding.
pub trait RegField<T>
where
  Self: Sized,
  T: RegTag,
{
  /// Parent register type.
  type Reg: Reg<T>;

  /// Address offset of the field.
  const OFFSET: usize;

  /// Bit-width of the field.
  const WIDTH: usize;

  /// Creates a new field binding.
  ///
  /// # Safety
  ///
  /// Shouldn't be called directly.
  unsafe fn bind() -> Self;
}

/// Single-bit register field.
pub trait RegFieldBit<T>
where
  Self: RegField<T>,
  T: RegTag,
{
}

/// Multiple-bits register field.
pub trait RegFieldBits<T>
where
  Self: RegField<T>,
  T: RegTag,
{
}

/// Synchronous register field.
pub trait SRegField
where
  Self: RegField<Srt>,
  Self::Reg: Reg<Srt>,
{
  /// Less strict type.
  type UpRegField: RegField<Drt>;

  /// Converts to a less strict type.
  fn upgrade(self) -> Self::UpRegField;
}

/// Duplicable register field.
pub trait DRegField
where
  Self: RegField<Drt>,
  Self::Reg: Reg<Drt>,
{
  /// Less strict type.
  type UpRegField: RegField<Crt>;

  /// Converts to a less strict type.
  fn upgrade(self) -> Self::UpRegField;

  /// Returns a copy of the register field.
  fn clone(&mut self) -> Self;
}

/// Register field that can read its value.
pub trait RRegField<T>
where
  Self: RegField<T>,
  Self::Reg: RReg<T>,
  T: RegTag,
{
  /// Reads a register value from its memory address.
  #[inline(always)]
  fn load_val(&self) -> <Self::Reg as Reg<T>>::Val {
    unsafe {
      <Self::Reg as Reg<T>>::Val::from_raw(read_volatile(
        Self::Reg::ADDRESS
          as *const <<Self::Reg as Reg<T>>::Val as RegVal>::Raw,
      ))
    }
  }
}

/// Register field that can write its value.
pub trait WRegField<T>
where
  Self: RegField<T>,
  Self::Reg: WReg<T>,
  T: RegTag,
{
}

/// Register field that can only read its value.
pub trait RoRegField<T>
where
  Self: RRegField<T>,
  Self::Reg: RReg<T>,
  T: RegTag,
{
}

/// Register field that can only write its value.
pub trait WoRegField<T>
where
  Self: WRegField<T>,
  Self::Reg: WReg<T>,
  T: RegTag,
{
}

/// Write-only field of write-only register.
pub trait WoWoRegField<T>
where
  Self: WoRegField<T>,
  Self::Reg: WoReg<T>,
  T: RegTag,
{
  /// Creates a new reset value.
  fn default_val(&self) -> <Self::Reg as Reg<T>>::Val;

  /// Writes the value `val`.
  fn store_val(&self, val: <Self::Reg as Reg<T>>::Val);

  /// Updates a new reset value with `f` and writes the result to the register's
  /// memory address.
  fn reset<F>(&self, f: F)
  where
    F: Fn(&mut <Self::Reg as Reg<T>>::Val);
}

/// Single-bit register field that can read its value.
pub trait RRegFieldBit<T>
where
  Self: RegFieldBit<T> + RRegField<T>,
  Self::Reg: RReg<T>,
  T: RegTag,
{
  /// Reads the state of the bit from `val`.
  fn read(&self, val: &<Self::Reg as Reg<T>>::Val) -> bool;

  /// Reads the state of the bit from memory.
  fn read_bit(&self) -> bool;
}

/// Single-bit register field that can write its value.
pub trait WRegFieldBit<T>
where
  Self: RegFieldBit<T> + WRegField<T>,
  Self::Reg: WReg<T>,
  T: RegTag,
{
  /// Sets the bit in `val`.
  fn set(&self, val: &mut <Self::Reg as Reg<T>>::Val);

  /// Clears the bit in `val`.
  fn clear(&self, val: &mut <Self::Reg as Reg<T>>::Val);

  /// Toggles the bit in `val`.
  fn toggle(&self, val: &mut <Self::Reg as Reg<T>>::Val);
}

/// Single-bit write-only field of write-only register.
pub trait WoWoRegFieldBit<T>
where
  Self: RegFieldBit<T> + WoRegField<T>,
  Self::Reg: WoReg<T>,
  T: RegTag,
{
  /// Sets the bit in memory.
  fn set_bit(&self);

  /// Clears the bit in memory.
  fn clear_bit(&self);

  /// Toggles the bit in memory.
  fn toggle_bit(&self);
}

/// Multiple-bits register field that can read its value.
pub trait RRegFieldBits<T>
where
  Self: RegFieldBits<T> + RRegField<T>,
  Self::Reg: RReg<T>,
  T: RegTag,
{
  /// Reads the bits from `val`.
  fn read(
    &self,
    val: &<Self::Reg as Reg<T>>::Val,
  ) -> <<Self::Reg as Reg<T>>::Val as RegVal>::Raw;

  /// Reads the bits from memory.
  fn read_bits(&self) -> <<Self::Reg as Reg<T>>::Val as RegVal>::Raw;
}

/// Multiple-bits register field that can write its value.
pub trait WRegFieldBits<T>
where
  Self: RegFieldBits<T> + WRegField<T>,
  Self::Reg: WReg<T>,
  T: RegTag,
{
  /// Write `bits` to `val`.
  fn write(
    &self,
    val: &mut <Self::Reg as Reg<T>>::Val,
    bits: <<Self::Reg as Reg<T>>::Val as RegVal>::Raw,
  );
}

/// Multiple-bits write-only field of write-only register.
pub trait WoWoRegFieldBits<T>
where
  Self: RegFieldBits<T> + WoRegField<T>,
  Self::Reg: WoReg<T>,
  T: RegTag,
{
  /// Sets the bit in memory.
  fn write_bits(&self, bits: <<Self::Reg as Reg<T>>::Val as RegVal>::Raw);
}

impl<T, U> WoWoRegField<T> for U
where
  T: RegTag,
  U: WoRegField<T>,
  U::Reg: WoReg<T>,
{
  #[inline(always)]
  fn default_val(&self) -> <U::Reg as Reg<T>>::Val {
    unsafe { <U::Reg as Reg<T>>::Val::reset() }
  }

  #[inline(always)]
  fn store_val(&self, val: <U::Reg as Reg<T>>::Val) {
    unsafe {
      write_volatile(
        U::Reg::ADDRESS as *mut <<U::Reg as Reg<T>>::Val as RegVal>::Raw,
        val.raw(),
      );
    }
  }

  #[inline(always)]
  fn reset<F>(&self, f: F)
  where
    F: Fn(&mut <U::Reg as Reg<T>>::Val),
  {
    let mut val = self.default_val();
    f(&mut val);
    self.store_val(val);
  }
}

impl<T, U> RRegFieldBit<T> for U
where
  T: RegTag,
  U: RegFieldBit<T> + RRegField<T>,
  U::Reg: RReg<T>,
{
  #[inline(always)]
  fn read(&self, val: &<U::Reg as Reg<T>>::Val) -> bool {
    unsafe {
      val.read_bit(<<U::Reg as Reg<T>>::Val as RegVal>::Raw::from_usize(
        U::OFFSET,
      ))
    }
  }

  #[inline(always)]
  fn read_bit(&self) -> bool {
    self.read(&self.load_val())
  }
}

impl<T, U> WRegFieldBit<T> for U
where
  T: RegTag,
  U: RegFieldBit<T> + WRegField<T>,
  U::Reg: WReg<T>,
{
  #[inline(always)]
  fn set(&self, val: &mut <U::Reg as Reg<T>>::Val) {
    unsafe {
      val.set_bit(<<U::Reg as Reg<T>>::Val as RegVal>::Raw::from_usize(
        U::OFFSET,
      ));
    }
  }

  #[inline(always)]
  fn clear(&self, val: &mut <U::Reg as Reg<T>>::Val) {
    unsafe {
      val.clear_bit(<<U::Reg as Reg<T>>::Val as RegVal>::Raw::from_usize(
        U::OFFSET,
      ));
    }
  }

  #[inline(always)]
  fn toggle(&self, val: &mut <U::Reg as Reg<T>>::Val) {
    unsafe {
      val.toggle_bit(<<U::Reg as Reg<T>>::Val as RegVal>::Raw::from_usize(
        U::OFFSET,
      ));
    }
  }
}

impl<T, U> WoWoRegFieldBit<T> for U
where
  T: RegTag,
  U: RegFieldBit<T> + WoRegField<T>,
  U::Reg: WoReg<T>,
{
  #[inline(always)]
  fn set_bit(&self) {
    self.reset(|val| {
      self.set(val);
    });
  }

  #[inline(always)]
  fn clear_bit(&self) {
    self.reset(|val| {
      self.clear(val);
    });
  }

  #[inline(always)]
  fn toggle_bit(&self) {
    self.reset(|val| {
      self.toggle(val);
    });
  }
}

impl<T, U> RRegFieldBits<T> for U
where
  T: RegTag,
  U: RegFieldBits<T> + RRegField<T>,
  U::Reg: RReg<T>,
{
  #[inline(always)]
  fn read(
    &self,
    val: &<U::Reg as Reg<T>>::Val,
  ) -> <<U::Reg as Reg<T>>::Val as RegVal>::Raw {
    unsafe {
      val.read_bits(
        <<U::Reg as Reg<T>>::Val as RegVal>::Raw::from_usize(U::OFFSET),
        <<U::Reg as Reg<T>>::Val as RegVal>::Raw::from_usize(U::WIDTH),
      )
    }
  }

  #[inline(always)]
  fn read_bits(&self) -> <<U::Reg as Reg<T>>::Val as RegVal>::Raw {
    self.read(&self.load_val())
  }
}

impl<T, U> WRegFieldBits<T> for U
where
  T: RegTag,
  U: RegFieldBits<T> + WRegField<T>,
  U::Reg: WReg<T>,
{
  #[inline(always)]
  fn write(
    &self,
    val: &mut <U::Reg as Reg<T>>::Val,
    bits: <<U::Reg as Reg<T>>::Val as RegVal>::Raw,
  ) {
    unsafe {
      val.write_bits(
        <<U::Reg as Reg<T>>::Val as RegVal>::Raw::from_usize(U::OFFSET),
        <<U::Reg as Reg<T>>::Val as RegVal>::Raw::from_usize(U::WIDTH),
        bits,
      );
    }
  }
}

impl<T, U> WoWoRegFieldBits<T> for U
where
  T: RegTag,
  U: RegFieldBits<T> + WoRegField<T>,
  U::Reg: WoReg<T>,
{
  #[inline(always)]
  fn write_bits(&self, bits: <<U::Reg as Reg<T>>::Val as RegVal>::Raw) {
    self.reset(|val| {
      self.write(val, bits);
    });
  }
}
