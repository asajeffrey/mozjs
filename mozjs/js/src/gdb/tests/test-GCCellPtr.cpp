#include "gdb-tests.h"
#include "jsapi.h"

#include "js/HeapAPI.h"
#include "js/Symbol.h"

FRAGMENT(GCCellPtr, simple) {
  JS::GCCellPtr nulll(nullptr);

  JS::Rooted<JSObject*> glob(cx, JS::CurrentGlobalOrNull(cx));
  JS::Rooted<JSString*> empty(cx, JS_NewStringCopyN(cx, nullptr, 0));
  JS::Rooted<JS::Symbol*> unique(cx, JS::NewSymbol(cx, nullptr));

  JS::GCCellPtr object(glob.get());
  JS::GCCellPtr string(empty.get());
  JS::GCCellPtr symbol(unique.get());

  breakpoint();

  use(nulll);
  use(object);
  use(string);
  use(symbol);
}
