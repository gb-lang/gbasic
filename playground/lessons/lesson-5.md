# Score and text

Goal: use variables and positioned text to show a score.

Text with `.at(x, y)` draws on the canvas instead of only printing to the console.

## Starter Code

```gbasic
let score = 0
let star = sprite("star")

while true {
    clear(20, 20, 40)
    Screen.SpriteAt(star, 360, 220)
    Screen.SpriteScale(star, 2.0)
    Screen.SpriteDraw(star)
    print("Score: {score}").at(20, 30)
}
```

## Solution Code

```gbasic
let score = 0
let star = sprite("star")

while true {
    clear(20, 20, 40)
    if key("space") {
        score = score + 1
        play("coin")
    }
    Screen.SpriteAt(star, 360, 220)
    Screen.SpriteScale(star, 2.0)
    Screen.SpriteDraw(star)
    print("Score: {score}").at(20, 30)
}
```
