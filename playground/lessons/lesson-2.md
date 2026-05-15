# Make it move

Goal: create a shape and let the runtime move it every frame.

Objects have properties like `.position`, `.velocity`, `.color`, and `.bounces`.

## Starter Code

```gbasic
let ball = circle(18)
ball.position = point(100, 100)
ball.color = yellow
ball.velocity = point(3, 2)
ball.bounces = true

while true {
    clear(20, 24, 38)
}
```

## Solution Code

```gbasic
let ball = circle(18)
ball.position = point(100, 100)
ball.color = yellow
ball.velocity = point(3, 2)
ball.bounces = true

while true {
    clear(20, 24, 38)
    print("Moving!").at(20, 30)
}
```
