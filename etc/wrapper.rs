/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

impl<T> ::std::ops::Deref for JS::Handle<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<T> ::std::ops::Deref for JS::MutableHandle<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        unsafe { &*self.ptr }
    }
}

impl<T> ::std::ops::DerefMut for JS::MutableHandle<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.ptr }
    }
}

impl Default for jsid {
    fn default() -> Self { unsafe { JSID_VOID } }
}

impl Default for JS::Value {
    fn default() -> Self {
        unsafe {
            let mut result: JS::Value = ::std::mem::zeroed();
            result.setUndefined();
            result
        }
    }
}

impl Default for JS::CompartmentOptions {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

impl Drop for JSAutoCompartment {
    fn drop(&mut self) {
        unsafe { JS_LeaveCompartment(self.cx_, self.oldCompartment_); }
    }
}


impl<T> JS::Handle<T> {
    pub fn get(&self) -> T
        where T: Copy
    {
        unsafe { *self.ptr }
    }

    pub unsafe fn from_marked_location(ptr: *const T) -> JS::Handle<T> {
        JS::Handle {
            ptr: ptr as *mut T,
            _phantom_0: ::std::marker::PhantomData,
        }
    }
}

impl<T> JS::MutableHandle<T> {
    pub unsafe fn from_marked_location(ptr: *mut T) -> JS::MutableHandle<T> {
        JS::MutableHandle {
            ptr: ptr,
            _phantom_0: ::std::marker::PhantomData,
        }
    }

    pub fn handle(&self) -> JS::Handle<T> {
        unsafe {
            JS::Handle::from_marked_location(self.ptr as *const _)
        }
    }

    pub fn get(&self) -> T
        where T: Copy
    {
        unsafe { *self.ptr }
    }

    pub fn set(&self, v: T)
        where T: Copy
    {
        unsafe { *self.ptr = v }
    }
}

impl JS::HandleValue {
    pub fn null() -> JS::HandleValue {
        unsafe {
            JS::NullHandleValue
        }
    }

    pub fn undefined() -> JS::HandleValue {
        unsafe {
            JS::UndefinedHandleValue
        }
    }
}

impl JS::HandleValueArray {
    pub unsafe fn from_rooted_slice(values: &[JS::Value]) -> JS::HandleValueArray {
        JS::HandleValueArray {
            length_: values.len(),
            elements_: values.as_ptr()
        }
    }
}

const NULL_OBJECT: *mut JSObject = 0 as *mut JSObject;

impl JS::HandleObject {
    pub fn null() -> JS::HandleObject {
        unsafe {
            JS::HandleObject::from_marked_location(&NULL_OBJECT)
        }
    }
}

// ___________________________________________________________________________
// Implementations for various things in jsapi.rs

impl JSJitMethodCallArgs {
    #[inline]
    pub fn get(&self, i: u32) -> JS::HandleValue {
        unsafe {
            if i < self.argc_ {
                JS::HandleValue::from_marked_location(self.argv_.offset(i as isize))
            } else {
                JS::UndefinedHandleValue
            }
        }
    }

    #[inline]
    pub fn index(&self, i: u32) -> JS::HandleValue {
        assert!(i < self.argc_);
        unsafe {
            JS::HandleValue::from_marked_location(self.argv_.offset(i as isize))
        }
    }

    #[inline]
    pub fn index_mut(&self, i: u32) -> JS::MutableHandleValue {
        assert!(i < self.argc_);
        unsafe {
            JS::MutableHandleValue::from_marked_location(self.argv_.offset(i as isize))
        }
    }
}

// XXX need to hack up bindgen to convert this better so we don't have
//     to duplicate so much code here
impl JS::CallArgs {
    #[inline]
    pub fn index(&self, i: u32) -> JS::HandleValue {
        assert!(i < self.argc_);
        unsafe {
            JS::HandleValue::from_marked_location(self.argv_.offset(i as isize))
        }
    }

    #[inline]
    pub fn index_mut(&self, i: u32) -> JS::MutableHandleValue {
        assert!(i < self.argc_);
        unsafe {
            JS::MutableHandleValue::from_marked_location(self.argv_.offset(i as isize))
        }
    }

    #[inline]
    pub fn get(&self, i: u32) -> JS::HandleValue {
        unsafe {
            if i < self.argc_ {
                JS::HandleValue::from_marked_location(self.argv_.offset(i as isize))
            } else {
                JS::UndefinedHandleValue
            }
        }
    }

    #[inline]
    pub fn rval(&self) -> JS::MutableHandleValue {
        unsafe {
            JS::MutableHandleValue::from_marked_location(self.argv_.offset(-2))
        }
    }

    #[inline]
    pub fn thisv(&self) -> JS::HandleValue {
        unsafe {
            JS::HandleValue::from_marked_location(self.argv_.offset(-1))
        }
    }

    #[inline]
    pub fn calleev(&self) -> JS::HandleValue {
        unsafe {
            JS::HandleValue::from_marked_location(self.argv_.offset(-2))
        }
    }

    #[inline]
    pub fn callee(&self) -> *mut JSObject {
        unsafe { self.calleev().toObject() }
    }

    #[inline]
    pub fn new_target(&self) -> JS::MutableHandleValue {
        assert!(self.constructing_());
        unsafe {
            JS::MutableHandleValue::from_marked_location(self.argv_.offset(self.argc_ as isize))
        }
    }
}

impl JSJitSetterCallArgs {
    #[inline]
    pub fn get(&self, i: u32) -> JS::HandleValue {
        assert!(i == 0);
        self._base.handle()
    }
}

impl JSNativeWrapper {
    pub fn is_zeroed(&self) -> bool {
        let JSNativeWrapper { op, info } = *self;
        op.is_none() && info.is_null()
    }
}
