/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use jsapi::JS;
use jsapi::jsid;
use jsapi::JSFunction;
use jsapi::JSFlatString;
use jsapi::JSObject;
use jsapi::JSString;
use jsapi::JSScript;

// Rooting API for standard JS things

pub trait RootKind {
    #[allow(non_snake_case)]
    #[inline(always)]
    fn rootKind() -> JS::RootKind;
}

impl RootKind for *mut JSObject {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::Object }
}

impl RootKind for *mut JSFlatString {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::String }
}

impl RootKind for *mut JSFunction {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::Object }
}

impl RootKind for *mut JSString {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::String }
}

impl RootKind for *mut JS::Symbol {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::Symbol }
}

impl RootKind for *mut JSScript {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::Script }
}

impl RootKind for jsid {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::Id }
}

impl RootKind for JS::Value {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::Value }
}

impl RootKind for JS::PropertyDescriptor {
    #[inline(always)]
    fn rootKind() -> JS::RootKind { JS::RootKind::Traceable }
}
