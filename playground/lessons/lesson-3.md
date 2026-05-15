# Listen for keys

Goal: use `key("left")` and `key("right")` to control a paddle.

The canvas needs focus for keyboard input, so click the game area after pressing Run.

## Starter Code

```gbasic
let paddle = rect(120, 20)
paddle.position = point(340, 540)
paddle.color = blue

while true {
    clear(15, 20, 35)
    if key("left") { paddle.position.x = paddle.position.x - 6 }
    if key("right") { paddle.position.x = paddle.position.x + 6 }
}
```

## Solution Code

```gbasic
let paddle = rect(120, 20)
paddle.position = point(340, 540)
paddle.color = blue

while true {
    clear(15, 20, 35)
    if key("left") { paddle.position.x = paddle.position.x - 6 }
    if key("right") { paddle.position.x = paddle.position.x + 6 }
    print("Use left and right").at(20, 30)
}
```
