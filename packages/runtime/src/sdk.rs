#![allow(non_snake_case)]

use core::ffi::c_double;

use vex_sdk::*;
use vexide::{core::println, prelude::Display};
use wasm3::{error::Trap, store::AsContextMut, Instance, Store};

use crate::{platform::draw_error, teavm::get_cstring, Data};

macro_rules! link {
    ($instance:ident, $store:ident, mod $module:literal {
        $( fn $name:ident ( $($arg:ident: $arg_ty:ty $(as $wrapper:expr)? $(,)?),* )  $(-> $ret:ty $(, in .$field:tt)?)?; )*
    }) => {
        {
            $(
                $instance.link_closure(
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
                )?;
            )*
        }
    };
}

macro_rules! printf_style {
    ($instance:ident, $store:ident, mod $module:literal {
        $( fn $name:ident ( $($arg:ident: $arg_ty:ty,)* @printf@); )*
    }) => {
        {
            $(
                $instance.link_closure(
                    &mut *$store,
                    $module,
                    stringify!($name),
                    #[allow(unused_parens)]
                    |mut ctx, ($($arg,)* string): ($($arg_ty,)* i32)| {
                        let string = get_cstring(&mut ctx, string);
                        unsafe {
                            vex_sdk::$name(
                                $($arg,)*
                                c"%s".as_ptr(),
                                string.as_ptr(),
                            );
                        }
                        Ok(())
                    }
                )?;
            )*
        }
    };
}

pub fn link(store: &mut Store<Data>, instance: &mut Instance<Data>) -> anyhow::Result<()> {
    instance.link_closure(
        &mut *store,
        "hydrozoa",
        "panic",
        |mut ctx, string: i32| -> Result<(), Trap> {
            let string = get_cstring(&mut ctx, string);
            let msg = string.to_string_lossy();

            let mut display = unsafe { Display::new() };

            draw_error(&mut display, msg.as_ref());

            loop {
                unsafe {
                    vex_sdk::vexTasksRun();
                }
            }
        },
    )?;

    instance.link_closure(
        &mut *store,
        "hydrozoa",
        "getByteArrayPointer",
        |mut ctx, (address, size): (i32, i32)| {
            let teavm = ctx.data().teavm.clone().unwrap();
            let array_ptr = (teavm.byte_array_data)(ctx.as_context_mut(), address).unwrap();
            let memory = ctx.memory_mut();
            Ok(memory.as_mut_ptr().wrapping_offset(array_ptr) as i32)
        },
    )?;

    link!(instance, store, mod "vex" {
        // System
        fn vexSystemTimeGet() -> u32;
        fn vexSystemExitRequest();
        fn vexSystemHighResTimeGet() -> u64;
        fn vexSystemPowerupTimeGet() -> u64;
        fn vexSystemLinkAddrGet() -> u32;
        fn vexSystemVersion() -> u32;
        fn vexStdlibVersion() -> u32;

        // Misc
        fn vexTasksRun();
        fn vexCompetitionStatus() -> u32;

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
        fn vexDisplayForegroundColorGet() -> u32;
        fn vexDisplayBackgroundColorGet() -> u32;
        fn vexDisplayClipRegionSet(x1: i32, y1: i32, x2: i32, y2: i32);
        fn vexDisplayRender(bVsyncWait: bool, bRunScheduler: bool);
        fn vexDisplayDoubleBufferDisable();
        fn vexDisplayClipRegionSetWithIndex(index: i32, x1: i32, y1: i32, x2: i32, y2: i32);
        // fn vexImageBmpRead(ibuf: *const u8, oBuf: *mut v5_image, maxw: u32, maxh: u32) -> u32;
        // fn vexImagePngRead(ibuf: *const u8, oBuf: *mut v5_image, maxw: u32, maxh: u32, ibuflen: u32) -> u32;
    });

    printf_style!(instance, store, mod "vex" {
        fn vexDisplayPrintf(xpos: i32, ypos: i32, bOpaque: i32, @printf@);
        fn vexDisplayString(nLineNumber: i32, @printf@);
        fn vexDisplayStringAt(xpos: i32, ypos: i32, @printf@);
        fn vexDisplayBigString(nLineNumber: i32, @printf@);
        fn vexDisplayBigStringAt(xpos: i32, ypos: i32, @printf@);
        fn vexDisplaySmallStringAt(xpos: i32, ypos: i32, @printf@);
        fn vexDisplayCenteredString(nLineNumber: i32, @printf@);
        fn vexDisplayBigCenteredString(nLineNumber: i32, @printf@);
    });

    link!(instance, store, mod "vex" {
    // AbsEnc
    fn vexDeviceAbsEncReset(device: u32);
    fn vexDeviceAbsEncPositionSet(device: u32, position: i32);
    fn vexDeviceAbsEncPositionGet(device: u32) -> i32;
    fn vexDeviceAbsEncVelocityGet(device: u32) -> i32;
    fn vexDeviceAbsEncAngleGet(device: u32) -> i32;
    fn vexDeviceAbsEncReverseFlagSet(device: u32, value: bool);
    fn vexDeviceAbsEncReverseFlagGet(device: u32) -> bool;
    fn vexDeviceAbsEncStatusGet(device: u32) -> u32;
    fn vexDeviceAbsEncDataRateSet(device: u32, rate: u32);
    // Adi
    fn vexDeviceAdiPortConfigSet(device: u32, port: u32, config: u32 as V5_AdiPortConfiguration);
    fn vexDeviceAdiPortConfigGet(device: u32, port: u32) -> u32, in .0;
    fn vexDeviceAdiValueSet(device: u32, port: u32, value: i32);
    fn vexDeviceAdiValueGet(device: u32, port: u32) -> i32;
    fn vexDeviceAdiAddrLedSet(device: u32, port: u32, pData: u32, nOffset: u32, nLength: u32, options: u32);
    fn vexDeviceBumperGet(device: u32) -> u32, in .0;
    fn vexDeviceGyroReset(device: u32);
    fn vexDeviceGyroHeadingGet(device: u32) -> c_double;
    fn vexDeviceGyroDegreesGet(device: u32) -> c_double;
    fn vexDeviceSonarValueGet(device: u32) -> i32;
    // AiVision
    fn vexDeviceAiVisionClassNameGet(device: u32, id: i32, pName: u32) -> i32;
    fn vexDeviceAiVisionCodeGet(device: u32, id: u32, pCode: u32) -> bool;
    fn vexDeviceAiVisionCodeSet(device: u32, pCode: u32);
    fn vexDeviceAiVisionColorGet(device: u32, id: u32, pColor: u32) -> bool;
    fn vexDeviceAiVisionColorSet(device: u32, pColor: u32);
    fn vexDeviceAiVisionModeGet(device: u32) -> u32;
    fn vexDeviceAiVisionModeSet(device: u32, mode: u32);
    fn vexDeviceAiVisionObjectCountGet(device: u32) -> i32;
    fn vexDeviceAiVisionObjectGet(device: u32, indexObj: u32, pObject: u32) -> i32;
    fn vexDeviceAiVisionSensorSet(device: u32, brightness: c_double, contrast: c_double);
    fn vexDeviceAiVisionStatusGet(device: u32) -> u32;
    fn vexDeviceAiVisionTemperatureGet(device: u32) -> c_double;
    // Arm
    fn vexDeviceArmMoveTipCommandLinearAdv(device: u32, position: u32, j6_rotation: c_double, j6_velocity: u32, relative: bool);
    fn vexDeviceArmMoveTipCommandJointAdv(device: u32, position: u32, j6_rotation: c_double, j6_velocity: u32, relative: bool);
    fn vexDeviceArmTipPositionGetAdv(device: u32, position: u32);
    fn vexDeviceArmPoseSet(device: u32, pose: u32, velocity: u32);
    fn vexDeviceArmMoveTipCommandLinear(device: u32, x: i32, y: i32, z: i32, pose: u32, velocity: u32, rotation: c_double, rot_velocity: u32, relative: bool);
    fn vexDeviceArmMoveTipCommandJoint(device: u32, x: i32, y: i32, z: i32, pose: u32, velocity: u32, rotation: c_double, rot_velocity: u32, relative: bool);
    fn vexDeviceArmMoveJointsCommand(device: u32, positions: u32, velocities: u32, j6_rotation: c_double, j6_velocity: u32, j7_volts: c_double, j7_timeout: u32, j7_i_limit: u32, relative: bool);
    fn vexDeviceArmSpinJoints(device: u32, velocities: u32);
    fn vexDeviceArmSetJointPositions(device: u32, new_positions: u32);
    fn vexDeviceArmPickUpCommand(device: u32);
    fn vexDeviceArmDropCommand(device: u32);
    fn vexDeviceArmMoveVoltsCommand(device: u32, voltages: u32);
    fn vexDeviceArmFullStop(device: u32, brakeMode: u32);
    fn vexDeviceArmEnableProfiler(device: u32, enable: u32);
    fn vexDeviceArmProfilerVelocitySet(device: u32, linear_velocity: u32, joint_velocity: u32);
    fn vexDeviceArmSaveZeroValues(device: u32);
    fn vexDeviceArmForceZeroCommand(device: u32);
    fn vexDeviceArmClearZeroValues(device: u32);
    fn vexDeviceArmBootload(device: u32);
    fn vexDeviceArmTipPositionGet(device: u32, x: u32, y: u32, z: u32);
    fn vexDeviceArmJointInfoGet(device: u32, positions: u32, velocities: u32, currents: u32);
    fn vexDeviceArmJ6PositionGet(device: u32) -> c_double;
    fn vexDeviceArmBatteryGet(device: u32) -> i32;
    fn vexDeviceArmServoFlagsGet(device: u32, servoID: u32) -> i32;
    fn vexDeviceArmStatusGet(device: u32) -> u32;
    fn vexDeviceArmDebugGet(device: u32, id: i32) -> u32;
    fn vexDeviceArmJointErrorsGet(device: u32, errors: u32);
    fn vexDeviceArmJ6PositionSet(device: u32, position: u32);
    fn vexDeviceArmStopJointsCommand(device: u32, brakeModes: u32);
    fn vexDeviceArmReboot(device: u32);
    fn vexDeviceArmTipOffsetSet(device: u32, x: i32, y: i32, z: i32);
    // Battery
    fn vexBatteryVoltageGet() -> i32;
    fn vexBatteryCurrentGet() -> i32;
    fn vexBatteryTemperatureGet() -> c_double;
    fn vexBatteryCapacityGet() -> c_double;
    // Competition
    fn vexCompetitionStatus() -> u32;
    fn vexCompetitionControl(data: u32);
    // Controller
    fn vexControllerGet(id: u32 as V5_ControllerId, index: u32 as V5_ControllerIndex) -> i32;
    fn vexControllerConnectionStatusGet(id: u32 as V5_ControllerId) -> u32, in .0;
    fn vexControllerTextSet(id: u32, line: u32, col: u32, buf: u32) -> u32;
    // Device
    fn vexDevicesGetNumber() -> u32;
    fn vexDevicesGetNumberByType(device_type: u32 as V5_DeviceType) -> u32;
    fn vexDevicesGet() -> u32;
    fn vexDeviceGetByIndex(index: u32) -> u32;
    fn vexDeviceGetStatus(devices: u32) -> i32;
    fn vexDeviceGetTimestamp(device: u32) -> u32;
    fn vexDeviceGenericValueGet(device: u32) -> c_double;
    fn vexDeviceButtonStateGet() -> i32;
    // Distance
    fn vexDeviceDistanceDistanceGet(device: u32) -> u32;
    fn vexDeviceDistanceConfidenceGet(device: u32) -> u32;
    fn vexDeviceDistanceStatusGet(device: u32) -> u32;
    fn vexDeviceDistanceObjectSizeGet(device: u32) -> i32;
    fn vexDeviceDistanceObjectVelocityGet(device: u32) -> c_double;
    // File
    fn vexFileMountSD() -> u32, in .0;
    fn vexFileDirectoryGet(path: u32, buffer: u32, len: u32) -> u32, in .0;
    fn vexFileOpen(filename: u32, mode: u32) -> u32;
    fn vexFileOpenWrite(filename: u32) -> u32;
    fn vexFileOpenCreate(filename: u32) -> u32;
    fn vexFileClose(fdp: u32);
    fn vexFileWrite(buf: u32, size: u32, nItems: u32, fdp: u32) -> i32;
    fn vexFileSize(fdp: u32) -> i32;
    fn vexFileSeek(fdp: u32, offset: u32, whence: i32) -> u32, in .0;
    fn vexFileRead(buf: u32, size: u32, nItems: u32, fdp: u32) -> i32;
    fn vexFileDriveStatus(drive: u32) -> bool;
    fn vexFileTell(fdp: u32) -> i32;
    fn vexFileSync(fdp: u32);
    fn vexFileStatus(filename: u32) -> u32;
    // GenericRadio
    fn vexDeviceGenericRadioWriteFree(device: u32) -> i32;
    fn vexDeviceGenericRadioTransmit(device: u32, data: u32, size: u32) -> i32;
    fn vexDeviceGenericRadioReceiveAvail(device: u32) -> i32;
    fn vexDeviceGenericRadioReceive(device: u32, data: u32, size: u32) -> i32;
    fn vexDeviceGenericRadioLinkStatus(device: u32) -> bool;
    // GenericSerial
    fn vexDeviceGenericSerialEnable(device: u32, options: i32);
    fn vexDeviceGenericSerialBaudrate(device: u32, baudrate: i32);
    fn vexDeviceGenericSerialWriteChar(device: u32, c: u32) -> i32;
    fn vexDeviceGenericSerialWriteFree(device: u32) -> i32;
    fn vexDeviceGenericSerialTransmit(device: u32, buffer: u32, length: i32) -> i32;
    fn vexDeviceGenericSerialReadChar(device: u32) -> i32;
    fn vexDeviceGenericSerialPeekChar(device: u32) -> i32;
    fn vexDeviceGenericSerialReceiveAvail(device: u32) -> i32;
    fn vexDeviceGenericSerialReceive(device: u32, buffer: u32, length: i32) -> i32;
    fn vexDeviceGenericSerialFlush(device: u32);
    // Gps
    fn vexDeviceGpsReset(device: u32);
    fn vexDeviceGpsHeadingGet(device: u32) -> c_double;
    fn vexDeviceGpsDegreesGet(device: u32) -> c_double;
    fn vexDeviceGpsQuaternionGet(device: u32, data: u32);
    fn vexDeviceGpsAttitudeGet(device: u32, data: u32, bRaw: bool);
    fn vexDeviceGpsRawGyroGet(device: u32, data: u32);
    fn vexDeviceGpsRawAccelGet(device: u32, data: u32);
    fn vexDeviceGpsStatusGet(device: u32) -> u32;
    fn vexDeviceGpsModeSet(device: u32, mode: u32);
    fn vexDeviceGpsModeGet(device: u32) -> u32;
    fn vexDeviceGpsDataRateSet(device: u32, rate: u32);
    fn vexDeviceGpsOriginSet(device: u32, ox: c_double, oy: c_double);
    fn vexDeviceGpsOriginGet(device: u32, ox: u32, oy: u32);
    fn vexDeviceGpsRotationSet(device: u32, value: c_double);
    fn vexDeviceGpsRotationGet(device: u32) -> c_double;
    fn vexDeviceGpsInitialPositionSet(device: u32, initial_x: c_double, initial_y: c_double, initial_rotation: c_double);
    fn vexDeviceGpsErrorGet(device: u32) -> c_double;
    // Imu
    fn vexDeviceImuReset(device: u32);
    fn vexDeviceImuHeadingGet(device: u32) -> c_double;
    fn vexDeviceImuDegreesGet(device: u32) -> c_double;
    fn vexDeviceImuQuaternionGet(device: u32, data: u32);
    fn vexDeviceImuAttitudeGet(device: u32, data: u32);
    fn vexDeviceImuRawGyroGet(device: u32, data: u32);
    fn vexDeviceImuRawAccelGet(device: u32, data: u32);
    fn vexDeviceImuStatusGet(device: u32) -> u32;
    fn vexDeviceImuModeSet(device: u32, mode: u32);
    fn vexDeviceImuModeGet(device: u32) -> u32;
    fn vexDeviceImuDataRateSet(device: u32, rate: u32);
    // Led
    fn vexDeviceLedSet(device: u32, value: u32 as V5_DeviceLedColor);
    fn vexDeviceLedRgbSet(device: u32, color: u32);
    fn vexDeviceLedGet(device: u32) -> u32, in .0;
    fn vexDeviceLedRgbGet(device: u32) -> u32;
    // LightTower
    fn vexDeviceLightTowerBlinkSet(device: u32, select: u32, mask: u32, onTime: i32, offTime: i32);
    fn vexDeviceLightTowerColorSet(device: u32, color_id: u32, value: u32);
    fn vexDeviceLightTowerRgbGet(device: u32) -> u32;
    fn vexDeviceLightTowerRgbSet(device: u32, rgb_value: u32, xyw_value: u32);
    fn vexDeviceLightTowerStatusGet(device: u32) -> u32;
    fn vexDeviceLightTowerDebugGet(device: u32, id: i32) -> u32;
    fn vexDeviceLightTowerXywGet(device: u32) -> u32;
    // Magnet
    fn vexDeviceMagnetPowerSet(device: u32, value: i32, time: i32);
    fn vexDeviceMagnetPowerGet(device: u32) -> i32;
    fn vexDeviceMagnetPickup(device: u32, duration: u32 as V5_DeviceMagnetDuration);
    fn vexDeviceMagnetDrop(device: u32, duration: u32 as V5_DeviceMagnetDuration);
    fn vexDeviceMagnetTemperatureGet(device: u32) -> c_double;
    fn vexDeviceMagnetCurrentGet(device: u32) -> c_double;
    fn vexDeviceMagnetStatusGet(device: u32) -> u32;
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
    fn vexDeviceMotorPositionRawGet(device: u32, timestamp: u32) -> i32;
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
    fn vexDeviceMotorPositionPidSet(device: u32, pid: u32);
    fn vexDeviceMotorVelocityPidSet(device: u32, pid: u32);
    fn vexDeviceMotorExternalProfileSet(device: u32, position: c_double, velocity: i32);
    // Optical
    fn vexDeviceOpticalHueGet(device: u32) -> c_double;
    fn vexDeviceOpticalSatGet(device: u32) -> c_double;
    fn vexDeviceOpticalBrightnessGet(device: u32) -> c_double;
    fn vexDeviceOpticalProximityGet(device: u32) -> i32;
    fn vexDeviceOpticalRgbGet(device: u32, data: u32);
    fn vexDeviceOpticalLedPwmSet(device: u32, value: i32);
    fn vexDeviceOpticalLedPwmGet(device: u32) -> i32;
    fn vexDeviceOpticalStatusGet(device: u32) -> u32;
    fn vexDeviceOpticalRawGet(device: u32, data: u32);
    fn vexDeviceOpticalModeSet(device: u32, mode: u32);
    fn vexDeviceOpticalModeGet(device: u32) -> u32;
    fn vexDeviceOpticalGestureGet(device: u32, pData: u32) -> u32;
    fn vexDeviceOpticalGestureEnable(device: u32);
    fn vexDeviceOpticalGestureDisable(device: u32);
    fn vexDeviceOpticalProximityThreshold(device: u32, value: i32);
    fn vexDeviceOpticalIntegrationTimeSet(device: u32, timeMs: c_double);
    fn vexDeviceOpticalIntegrationTimeGet(device: u32) -> c_double;
    // Pneumatic
    fn vexDevicePneumaticActuationStatusGet(device: u32, ac1: u32, ac2: u32, ac3: u32, ac4: u32) -> u32;
    fn vexDevicePneumaticCompressorSet(device: u32, bState: bool);
    fn vexDevicePneumaticCtrlSet(device: u32, pCtrl: u32);
    fn vexDevicePneumaticCylinderPwmSet(device: u32, id: u32, bState: bool, pwm: u32);
    fn vexDevicePneumaticCylinderSet(device: u32, id: u32, bState: bool);
    fn vexDevicePneumaticPwmGet(device: u32) -> u32;
    fn vexDevicePneumaticPwmSet(device: u32, pwm: u32);
    fn vexDevicePneumaticStatusGet(device: u32) -> u32;
    // Range
    fn vexDeviceRangeValueGet(device: u32) -> i32;
    // Serial
    fn vexSerialWriteChar(channel: u32, c: u32) -> i32;
    fn vexSerialWriteBuffer(channel: u32, data: u32, data_len: u32) -> i32;
    fn vexSerialReadChar(channel: u32) -> i32;
    fn vexSerialPeekChar(channel: u32) -> i32;
    fn vexSerialWriteFree(channel: u32) -> i32;
    // Touch
    fn vexTouchDataGet(status: u32);
    });

    // instance.link_closure(
    //     &mut *store,
    //     "vex",
    //     "vexDeviceGetStatus",
    //     |mut ctx, devices| {
    //         let teavm = ctx.data().teavm.clone().unwrap();
    //         let array_ptr = (teavm.byte_array_data)(ctx.as_context_mut(), devices).unwrap();
    //         let memory = ctx.memory_mut();
    //         let devices = &mut memory
    //             [array_ptr as usize..(array_ptr as usize + vex_sdk::V5_MAX_DEVICE_PORTS)];

    //         let devices = unsafe {
    //             // SAFETY: V5_DeviceType is a repr(transparent) struct holding a u8
    // core::mem::transmute::<*mut u8, *mut V5_DeviceType>(devices.as_mut_ptr())
    //         };
    //         Ok(unsafe { vex_sdk::vexDeviceGetStatus(devices) })
    //     },
    // )?;

    instance.link_closure(
        &mut *store,
        "vex",
        "vexDisplayStringWidthGet",
        |mut ctx, string: i32| {
            let string = get_cstring(&mut ctx, string);
            Ok(unsafe { vex_sdk::vexDisplayStringWidthGet(string.as_ptr()) })
        },
    )?;

    instance.link_closure(
        &mut *store,
        "vex",
        "vexDisplayStringHeightGet",
        |mut ctx, string: i32| {
            let string = get_cstring(&mut ctx, string);
            Ok(unsafe { vex_sdk::vexDisplayStringHeightGet(string.as_ptr()) })
        },
    )?;

    instance.link_closure(
        &mut *store,
        "vex",
        "vexDisplayFontNamedSet",
        |mut ctx, string: i32| {
            let string = get_cstring(&mut ctx, string);
            unsafe { vex_sdk::vexDisplayFontNamedSet(string.as_ptr()) };
            Ok(())
        },
    )?;

    Ok(())
}
