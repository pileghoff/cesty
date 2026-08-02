
extern int bar(); // We mock this
extern int baz(); // We auto-stub this

int foo(int a) {
  if (a == 0) {
    return bar();
  }

  if (a == 1) {
    return baz();
  }

  return 0;
}
