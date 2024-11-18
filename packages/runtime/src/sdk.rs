#![allow(non_snake_case)]

use core::ffi::c_double;

use anyhow::Context;
use vex_sdk::{
    V5MotorBrakeMode, V5MotorControlMode, V5MotorEncoderUnits, V5MotorGearset, V5_ControllerId,
    V5_ControllerIndex, V5_Device, V5_DeviceT, V5_DeviceType,
};
use wasm3::{store::AsContextMut, Instance, Store};

use crate::{teavm::TeaVM, Data};

macro_rules! link {
    ($instance:ident, $store:ident, mod $module:literal {
        $( fn $name:ident ( $($arg:ident: $arg_ty:ty $(as $wrapper:expr)? $(,)?),* )  $(-> $ret:ty $(, in .$field:tt)?)? );* $(;)?
    }) => {
        {
            $(
                _ = $instance.link_closure(
                    &mut *$store,
                    $module,
                    stringify!($name),
                    #[allow(unused_parens)]
                    |_ctx, ($($arg),*): ($($arg_ty),*)| {
                        #[inline]
                        fn inner($($arg: $arg_ty),*) $(-> $ret)? {
                            unsafe {
                                vex_sdk::$name(
                                    $($($wrapper)? ($arg as _)),*
                                ) $($(.$field)? as $ret)?
                            }
                        }
                        Ok(inner($($arg),*))
                    }
                ).context(concat!("Unable to link ", $module, "::", stringify!($name), " function"));
            )*
        }
    };
}

pub fn link(store: &mut Store<Data>, instance: &mut Instance<Data>) -> anyhow::Result<()> {
    link!(instance, store, mod "vex" {
        // Display
        fn vexDisplayForegroundColor(col: u32);
        fn vexDisplayBackgroundColor(col: u32);
        fn vexDisplayErase();
        fn vexDisplayScroll(nStartLine: i32, nLines: i32);
        fn vexDisplayScrollRect(x1: i32, y1: i32, x2: i32, y2: i32, nLines: i32);
        // fn vexDisplayCopyRect(x1: i32, y1: i32, x2: i32, y2: i32, pSrc: *mut u32, srcStride: i32);
        fn vexDisplayPixelSet(x: u32, y: u32);
        fn vexDisplayPixelClear(x: u32, y: u32);
        fn vexDisplayLineDraw(x1: i32, y1: i32, x2: i32, y2: i32);
        fn vexDisplayLineClear(x1: i32, y1: i32, x2: i32, y2: i32);
        fn vexDisplayRectDraw(x1: i32, y1: i32, x2: i32, y2: i32);
        fn vexDisplayRectClear(x1: i32, y1: i32, x2: i32, y2: i32);
        fn vexDisplayRectFill(x1: i32, y1: i32, x2: i32, y2: i32);
        fn vexDisplayCircleDraw(xc: i32, yc: i32, radius: i32);
        fn vexDisplayCircleClear(xc: i32, yc: i32, radius: i32);
        fn vexDisplayCircleFill(xc: i32, yc: i32, radius: i32);
        fn vexDisplayTextSize(n: u32, d: u32);
        // fn vexDisplayFontNamedSet(pFontName: *const c_char);
        fn vexDisplayForegroundColorGet() -> u32;
        fn vexDisplayBackgroundColorGet() -> u32;
        // fn vexDisplayStringWidthGet(pString: *const c_char) -> i32;
        // fn vexDisplayStringHeightGet(pString: *const c_char) -> i32;
        fn vexDisplayClipRegionSet(x1: i32, y1: i32, x2: i32, y2: i32);
        fn vexDisplayRender(bVsyncWait: bool, bRunScheduler: bool);
        fn vexDisplayDoubleBufferDisable();
        fn vexDisplayClipRegionSetWithIndex(index: i32, x1: i32, y1: i32, x2: i32, y2: i32);
        // fn vexImageBmpRead(ibuf: *const u8, oBuf: *mut v5_image, maxw: u32, maxh: u32) -> u32;
        // fn vexImagePngRead(ibuf: *const u8, oBuf: *mut v5_image, maxw: u32, maxh: u32, ibuflen: u32) -> u32;

        // fn vexDisplayVPrintf(xpos: i32, ypos: i32, bOpaque: i32, format: *const c_char, args: VaList);
        // fn vexDisplayVString(nLineNumber: i32, format: *const c_char, args: VaList);
        // fn vexDisplayVStringAt(xpos: i32, ypos: i32, format: *const c_char, args: VaList);
        // fn vexDisplayVBigString(nLineNumber: i32, format: *const c_char, args: VaList);
        // fn vexDisplayVBigStringAt(xpos: i32, ypos: i32, format: *const c_char, args: VaList);
        // fn vexDisplayVSmallStringAt(xpos: i32, ypos: i32, format: *const c_char, args: VaList);
        // fn vexDisplayVCenteredString(nLineNumber: i32, format: *const c_char, args: VaList);
        // fn vexDisplayVBigCenteredString(nLineNumber: i32, format: *const c_char, args: VaList);

        // Controller
        fn vexControllerGet(id: u32 as V5_ControllerId, index: u32 as V5_ControllerIndex) -> i32;
        fn vexControllerConnectionStatusGet(id: u32 as V5_ControllerId) -> u32, in .0;

        // Device
        fn vexDeviceGetByIndex(index: u32) -> u32;


        // Motor
        fn vexDeviceMotorVelocitySet(device: u32, velocity: i32);
        fn vexDeviceMotorVelocityGet(device: u32) -> i32;
        fn vexDeviceMotorActualVelocityGet(device: u32) -> c_double;
        fn vexDeviceMotorDirectionGet(device: u32) -> i32;
        fn vexDeviceMotorModeSet(device: u32, mode: u32 as V5MotorControlMode);
        fn vexDeviceMotorModeGet(device: u32) -> u32, in .0;
        fn vexDeviceMotorPwmSet(device: u32, pwm: i32);
        fn vexDeviceMotorPwmGet(device: u32) -> i32;
        fn vexDeviceMotorCurrentLimitSet(device: u32, limit: i32);
        fn vexDeviceMotorCurrentLimitGet(device: u32) -> i32;
        fn vexDeviceMotorCurrentGet(device: u32) -> i32;
        fn vexDeviceMotorPowerGet(device: u32) -> c_double;
        fn vexDeviceMotorTorqueGet(device: u32) -> c_double;
        fn vexDeviceMotorEfficiencyGet(device: u32) -> c_double;
        fn vexDeviceMotorTemperatureGet(device: u32) -> c_double;
        fn vexDeviceMotorOverTempFlagGet(device: u32) -> bool;
        fn vexDeviceMotorCurrentLimitFlagGet(device: u32) -> bool;
        fn vexDeviceMotorZeroVelocityFlagGet(device: u32) -> bool;
        fn vexDeviceMotorZeroPositionFlagGet(device: u32) -> bool;
        fn vexDeviceMotorReverseFlagSet(device: u32, reverse: bool);
        fn vexDeviceMotorReverseFlagGet(device: u32) -> bool;
        fn vexDeviceMotorEncoderUnitsSet(device: u32, units: u32 as V5MotorEncoderUnits);
        fn vexDeviceMotorEncoderUnitsGet(device: u32) -> u32, in .0;
        fn vexDeviceMotorBrakeModeSet(device: u32, mode: u32 as V5MotorBrakeMode);
        fn vexDeviceMotorBrakeModeGet(device: u32) -> u32, in .0;
        fn vexDeviceMotorPositionSet(device: u32, position: c_double);
        fn vexDeviceMotorPositionGet(device: u32) -> c_double;
        // fn vexDeviceMotorPositionRawGet(device: u32, timestamp: *mut u32) -> i32;
        fn vexDeviceMotorPositionReset(device: u32);
        fn vexDeviceMotorTargetGet(device: u32) -> c_double;
        fn vexDeviceMotorServoTargetSet(device: u32, position: c_double);
        fn vexDeviceMotorAbsoluteTargetSet(device: u32, position: c_double, veloctiy: i32);
        fn vexDeviceMotorRelativeTargetSet(device: u32, position: c_double, velocity: i32);
        fn vexDeviceMotorFaultsGet(device: u32) -> u32;
        fn vexDeviceMotorFlagsGet(device: u32) -> u32;
        fn vexDeviceMotorVoltageSet(device: u32, voltage: i32);
        fn vexDeviceMotorVoltageGet(device: u32) -> i32;
        fn vexDeviceMotorGearingSet(device: u32, gearset: u32 as V5MotorGearset);
        fn vexDeviceMotorGearingGet(device: u32) -> u32, in .0;
        fn vexDeviceMotorVoltageLimitSet(device: u32, limit: i32);
        fn vexDeviceMotorVoltageLimitGet(device: u32) -> i32;
        fn vexDeviceMotorVelocityUpdate(device: u32, velocity: i32);
        // fn vexDeviceMotorPositionPidSet(device: u32, pid: *mut V5_DeviceMotorPid);
        // fn vexDeviceMotorVelocityPidSet(device: u32, pid: *mut V5_DeviceMotorPid);
        fn vexDeviceMotorExternalProfileSet(device: u32, position: c_double, velocity: i32);

        // Misc
        fn vexTasksRun();
        fn vexCompetitionStatus() -> u32;
    });

    _ = instance
        .link_closure(
            &mut *store,
            "vex",
            "vexDeviceGetStatus",
            |mut ctx, devices: i32| {
                let teavm = ctx.data().teavm.clone().unwrap();
                let array_ptr = (teavm.byte_array_data)(ctx.as_context_mut(), devices).unwrap();
                let memory = ctx.memory_mut();
                let devices = &mut memory
                    [array_ptr as usize..(array_ptr as usize + vex_sdk::V5_MAX_DEVICE_PORTS)];

                let devices = unsafe {
                    // SAFETY: V5_DeviceType is a repr(transparent) struct holding a u8
                    core::mem::transmute::<*mut u8, *mut V5_DeviceType>(devices.as_mut_ptr())
                };
                Ok(unsafe { vex_sdk::vexDeviceGetStatus(devices) })
            },
        )
        .context("Unable to link vex::vexDeviceGetStatus function");

    Ok(())
}
