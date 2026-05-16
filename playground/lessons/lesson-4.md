# Bouncing

Goal: combine movement, bounces, and collision.

Make a ball bounce on the screen and a paddle.

## Starter Code

```gbasic
let ball = circle(14)
ball.position = point(120, 120)
ball.velocity = point(4, 3)
ball.color = yellow
ball.bounces = true

let paddle = rect(140, 20)
paddle.position = point(330, 540)
paddle.color = blue
paddle.solid = true

while true {
    clear(10, 16, 30)
    if key("left") { paddle.position.x = paddle.position.x - 7 }
    if key("right") { paddle.position.x = paddle.position.x + 7 }
}
```

## Solution Code

```gbasic
let ball = circle(14)
ball.position = point(120, 120)
ball.velocity = point(4, 3)
ball.color = yellow
ball.bounces = true

let paddle = rect(140, 20)
paddle.position = point(330, 540)
paddle.color = blue
paddle.solid = true

while true {
    clear(10, 16, 30)
    if key("left") { paddle.position.x = paddle.position.x - 7 }
    if key("right") { paddle.position.x = paddle.position.x + 7 }
    if ball.collides(paddle) { play("hit") }
}
```
