# gpui-transitions.

This crate provides an API for interpolating between values in [gpui](https://www.gpui.rs).


Transitions can be constructed via `window.use_transition` or `window.use_keyed_transition`. It's very similar to the `use_state` API.
```rs
let mut my_transition = window
    .use_keyed_transition(
        "my_transition",
        cx,
        Duration::from_millis(400),
        |_window, _cx| rgb(0xFF0000)),
    );
```

<br>

They can also be constructed more granularly via `Transition::new`:
```rs
let mut my_transition = Transition::new(
    cx.new(|_cx| TransitionState::new(rgb(0xFF0000))),
    Duration::from_millis(400),
);
```

<br>

To get the value of a transition you can use `evaluate`:

```rs
let value = my_transition.evaluate(window, cx);
```

If the transition is not finished when `evaluate` is called then an animation frame will be requested.

- - -

More examples can be found [here](/).
