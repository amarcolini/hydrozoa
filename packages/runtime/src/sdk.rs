#![allow(non_snake_case)]

use anyhow::Context;
use wasm3::{Instance, Store};

macro_rules! link {
    ($instance:ident, $store:ident, mod $module:literal {
        $( fn $name:ident ( $($arg:ident: $arg_ty:ty $(,)?),* )  $(-> $ret:ty)?);* $(;)?
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
                            unsafe { vex_sdk::$name($($arg),*) }
                        }
                        Ok(inner($($arg),*))
                    }
                ).context(concat!("Unable to link ", $module, "::", stringify!($name), " function"))?;
            )*
        }
    };
}

pub fn link(store: &mut Store, instance: &mut Instance) -> anyhow::Result<()> {
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

        // Misc
        fn vexTasksRun();
        fn vexCompetitionStatus() -> u32;
    });

    Ok(())
}
