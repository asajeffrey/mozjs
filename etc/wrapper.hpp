/* -*- Mode: C++; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 4 -*-
 * vim: set ts=8 sts=4 et sw=4 tw=99:
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include <stdint.h>
#ifndef _MSC_VER
#include <unistd.h>
#endif

typedef uint32_t HashNumber;

#include "jsfriendapi.h"
#include "js/Conversions.h"
#include "js/Initialization.h"
#include "js/MemoryMetrics.h"

// There's a couple of classes from pre-57 releases of SM that bindgen can't deal with.
// https://github.com/rust-lang-nursery/rust-bindgen/issues/851
// https://bugzilla.mozilla.org/show_bug.cgi?id=1277338
// https://rust-lang-nursery.github.io/rust-bindgen/replacing-types.html

/**
 * <div rustbindgen replaces="JS::CallArgs"></div>
 */

class MOZ_STACK_CLASS CallArgsReplacement
{
  protected:
    JS::Value* argv_;
    unsigned argc_;
    bool constructing_:1;
    bool ignoresReturnValue_:1;
#ifdef JS_DEBUG
    JS::detail::IncludeUsedRval wantUsedRval_;
#endif
};

/**
 * <div rustbindgen replaces="JSJitMethodCallArgs"></div>
 */

class JSJitMethodCallArgsReplacement
{
  private:
    JS::Value* argv_;
    unsigned argc_;
    bool constructing_:1;
    bool ignoresReturnValue_:1;
#ifdef JS_DEBUG
    JS::detail::NoUsedRval wantUsedRval_;
#endif
};
