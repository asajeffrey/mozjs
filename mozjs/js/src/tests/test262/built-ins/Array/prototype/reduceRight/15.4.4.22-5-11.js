// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.4.4.22-5-11
description: >
    Array.prototype.reduceRight - side-effects produced by step 3 when
    an exception occurs
---*/

        var obj = { 0: 11, 1: 12 };

        var accessed = false;

        Object.defineProperty(obj, "length", {
            get: function () {
                return {
                    toString: function () {
                        accessed = true;
                        return "0";
                    }
                };
            },
            configurable: true
        });
assert.throws(TypeError, function() {
            Array.prototype.reduceRight.call(obj, function () { });
});
assert(accessed, 'accessed !== true');

reportCompare(0, 0);