/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 * vim: set ts=8 sts=4 et sw=4 tw=99:
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "jsglue.hpp"

// Reexport some functions that are marked inline.

JS::CompartmentOptions JS_NewCompartmentOptions() {
    JS::CompartmentOptions result;
    return result;
}

JS::OwningCompileOptions JS_NewOwningCompileOptions(JSContext* cx) {
    JS::OwningCompileOptions result(cx);
    return result;
}

JS::shadow::Zone* JS_AsShadowZone(JS::Zone* zone) {
    return JS::shadow::Zone::asShadowZone(zone);
}

// These functions are only intended for use in testing,
// to make sure that the Rust implementation of JS::Value
// agrees with the C++ implementation.

JS::Value JS_Int32Value(int32_t value) {
    return JS::Int32Value(value);
}

bool JS_ValueIsInt32(JS::Value value) {
    return value.isInt32();
}

int32_t JS_ValueToInt32(JS::Value value) {
    return value.toInt32();
}
