# Function mocking

Cesty allows you to mock C based functions.
It done in two steps.

First you decalre the mock globally. This defines the name and type of function.

```rust
use cesty::define_mock;
define_mock!(fn foo(bar: c_int));
```

This defines a mock of the function `foo`, that takes a single argument `bar` of type int.

This should be declared in the global scope.

Inside the test you can create an instance of this mock the following way:

```rust
let foo_mock = mock!(foo);
```

The mock allows you to:

- Set the return values, either using `foo_mock.set_default_return` or `foo_mock.add_return`
- Provide a custom callback that is called when the function is invoked. This is done using `foo_mock.handler`
- Get all the calls to the function as a vec, using `foo_mock.calls`

Setting return values and setting a custom handler are not supported at the same time.

The idea of Cesty function mocking, is that we try to make it easy to follow the Arrange-Act-Assert pattern.

First you setup the mock.
For simple  function you can just tell the mock what values to return. For more complicated function that has to return something based on the input or through a pointer can use the custom handler.

Then you call the function under test.

Then you inspect the list of calls, to check that the function was actually called with the arguments you expected and in the order you expected.


## Setting up return values

When setting up return values you have two options.
First you can set a default return value.

Secondly you can add return values to a queue using `add_return`. If you call this function multiple times, the values will be returned in the order provided.

## Custom handlers

Custom handlers can be used instead of return values when more complex logic is needed.

To do this, you call `set_handler`, and provide a boxed FnMut. The input arguments will be a tuple of the input arguments to the mocked function.

Example:

For the C function `int baz(int a, bool b)`, the callback should take a tuple of type `(c_int, c_bool)` and return a `c_int`.

Often times you want to have some state that can be shared between mutliple callbacks as well as the test. For this you can use `Arc<Mutex<_>`, but Cesty also has a convient wrapper, called `shared_state<_>` for this exact purpose.

Example:

```rust
#[cesty_test]
fn c_driver_cutsom_handler() {
    let val = SharedState::new(vec![2]);
    let mut gpio_read = mock!(hal_gpio_read);
    {
        let val = val.clone();
        gpio_read.handler(Box::new(move |_| val.update(|v| v.pop().unwrap())));
    }

    assert_eq!(cesty_gpio_example::read_button(4), 2);
    val.update(|v| v.push(4));

    assert_eq!(cesty_gpio_example::read_button(4), 4);
}
```
