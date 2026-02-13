# G-Basic API Design Guardrails

**Purpose:** This document is the single source of truth for API design decisions in G-Basic. Every example, runtime function, parser feature, and intellisense suggestion must conform to these rules. AI-generated code that violates these guardrails is **wrong by definition**, regardless of whether it compiles.

**Core metric:** A beginner with reactive intellisense should be able to write a Pong game in under 10 minutes.

---

## Part 1: The Three Layers of Simplicity

G-Basic supports three progressively detailed ways to express the same operation. All three are valid, all three compile to the same IR, and intellisense guides the user from Layer 1 → Layer 2 → Layer 3 naturally.

### Layer 1: Vocabulary (Human-Readable Shortcuts)

Short, memorable words that read like English. These are **parser-level sugar** that desugar to Layer 3 before typechecking.

```
print("Hello!")
clear(0, 0, 0)
random(1, 10)
wait(2)
key("left")
play("bounce")
```

**Rules:**
- Every Layer 1 word must be a **single common English verb or noun** (no camelCase, no underscores)
- Maximum 1-3 arguments — if more are needed, the operation belongs at Layer 2+
- Layer 1 words are **always lowercase**
- They are chainable: `print("Score: {s}").at(10, 10)` works because the desugared form is a method chain
- The alias table is defined in `gbasic-common` — one line per alias, single source of truth

### Layer 2: Object-Oriented (Discoverable via Intellisense)

Create objects with properties, then manipulate them. This is the **primary teaching API** — where beginners spend most of their time after the first 5 minutes.

```
let paddle = Screen.Rect(100, 20)
paddle.position = Screen.center
paddle.color = white

let ball = Screen.Circle(10)
ball.position = Point(400, 300)
ball.color = yellow
```

**Rules:**
- Objects are created via `Namespace.Constructor(args)` — always returns a handle
- Properties are set via `.property = value` — reads like English
- Properties are read via `object.property` — no getters needed
- Method chains are for **actions**: `ball.move(3, 3)`, `paddle.draw()`
- Constructor arguments are **only dimensions or identity** (name, size) — never position, color, or behavior

### Layer 3: Full Namespace Chains (Expert / Explicit Control)

The complete, explicit form. Every Layer 1 and Layer 2 expression desugars to this.

```
Screen.Layer(0).Rect(100, 20).Position(350, 550).Color(255, 255, 255).Draw()
Screen.Layer(0).Circle(10).Position(400, 300).Color(255, 255, 0).Draw()
```

**Rules:**
- Always starts with a Namespace keyword (Screen, Sound, Input, Math, System, Memory, IO, Asset)
- Follows the **Entry → Target → Configuration → Action** pattern (see Part 3)
- Every method in the chain returns the builder object (except terminal actions)
- Terminal actions (`.Draw()`, `.Play()`, `.Start()`) return void or a result

---

## Part 2: Predefined Vocabulary (Built-In Constants)

G-Basic ships with a set of **predefined constants** that map to common values. These exist so beginners never need to type magic numbers for standard concepts.

### Positions (Screen Properties)

| Property | Meaning | Equivalent |
|----------|---------|------------|
| `Screen.center` | Center of screen | `Point(Screen.width / 2, Screen.height / 2)` |
| `Screen.top_left` | Top-left corner | `Point(0, 0)` |
| `Screen.top_right` | Top-right corner | `Point(Screen.width, 0)` |
| `Screen.bottom_left` | Bottom-left corner | `Point(0, Screen.height)` |
| `Screen.bottom_right` | Bottom-right corner | `Point(Screen.width, Screen.height)` |
| `Screen.top_center` | Top center | `Point(Screen.width / 2, 0)` |
| `Screen.bottom_center` | Bottom center | `Point(Screen.width / 2, Screen.height)` |

**Implementation:** These are **computed properties** on the `Screen` namespace — they resolve at runtime based on the current screen dimensions. They follow the property rule: `Screen.center` not `Screen.Center()`. Sub-fields are accessible: `Screen.center.y`, `Screen.bottom_center.x`.

### Colors (Named)

| Constant | RGB | Constant | RGB |
|----------|-----|----------|-----|
| `black` | (0, 0, 0) | `white` | (255, 255, 255) |
| `red` | (255, 0, 0) | `green` | (0, 255, 0) |
| `blue` | (0, 0, 255) | `yellow` | (255, 255, 0) |
| `orange` | (255, 165, 0) | `purple` | (128, 0, 128) |
| `pink` | (255, 192, 203) | `cyan` | (0, 255, 255) |
| `gray` | (128, 128, 128) | `brown` | (139, 69, 19) |

**Implementation:** Named colors are global constants of type `Color`. They can be used anywhere a color is expected: `paddle.color = white` or `Screen.Layer(0).Clear(black)`.

### Directions

| Constant | Value | Use Case |
|----------|-------|----------|
| `up` | (0, -1) | Movement vectors |
| `down` | (0, 1) | Movement vectors |
| `left` | (-1, 0) | Movement vectors |
| `right` | (1, 0) | Movement vectors |

**Implementation:** Direction constants are `Vector2` values. They can be scaled: `ball.velocity = right * 3 + down * 3`.

### Shapes (Constructors)

These are **shortcut constructors** — Layer 1 sugar for creating screen objects.

| Shortcut | Desugars To | Returns |
|----------|-------------|---------|
| `rect(w, h)` | `Screen.Rect(w, h)` | Object handle |
| `circle(r)` | `Screen.Circle(r)` | Object handle |
| `line(from, to)` | `Screen.Line(from, to)` | Object handle (from/to are Points) |
| `text(content)` | `Screen.Text(content)` | Object handle |
| `sprite(name)` | `Screen.Sprite(name)` | Object handle |

### Keys (String Constants)

Key names are plain strings, matching what's printed on the key:

```
key("left")    key("right")   key("up")      key("down")
key("space")   key("enter")   key("escape")
key("a")       key("b")       key("1")       key("2")
```

**Rule:** Key names are always lowercase strings. No `Key.LEFT` enum — strings are simpler for beginners and work with intellisense autocomplete.

### Value Types (Point, Color, Size)

G-Basic has three built-in **value types** that represent compound data. They are not objects (no handle, no runtime management) — they are plain values like `Int` or `Float`, just with named fields.

| Type | Constructor | Fields | Example |
|------|-------------|--------|---------|
| `Point` | `Point(x, y)` | `.x`, `.y` | `Point(100, 200)` |
| `Color` | `Color(r, g, b)` | `.r`, `.g`, `.b` | `Color(255, 0, 0)` |
| `Size` | `Size(w, h)` | `.width`, `.height` | `Size(100, 20)` |

**What they are:** Value constructors — the same category as writing `42` or `"hello"`. They create a value, not an object. No `new` keyword, no allocation, no handle.

**Tuple shorthand:** Where context makes the type unambiguous, a parenthesized pair can be used:

```
ball.position = (400, 300)          // shorthand for Point(400, 300)
ball.position = Point(400, 300)     // explicit — always works

paddle.color = (255, 255, 255)     // shorthand for Color(255, 255, 255)
paddle.color = white               // named constant — preferred
paddle.color = Color(255, 255, 255) // explicit — always works
```

**The Property Rule:** If something **is a value you read or write**, it's a **property** (no parentheses). If something **performs an action**, it's a **method** (with parentheses).

| Expression | Category | Why |
|------------|----------|-----|
| `Screen.width` | Property | Reading a value |
| `Screen.height` | Property | Reading a value |
| `Screen.title` | Property | Reading/writing a value |
| `paddle.position` | Property | Reading/writing a value |
| `paddle.position.x` | Property | Reading/writing a sub-value |
| `paddle.color` | Property | Reading/writing a value |
| `paddle.size.width` | Property | Reading a sub-value |
| `paddle.move(8, 0)` | Method | Performing an action |
| `paddle.collides(ball)` | Method | Performing a computation |
| `ball.remove()` | Method | Performing an action |
| `System.frame_time` | Property | Reading a value |
| `System.fps` | Property | Reading/writing a value |

**Rule:** Never put `()` on something that just returns a stored value. `Screen.width` not `Screen.Width()`. `System.frame_time` not `System.FrameTime()`.

---

## Part 3: The Namespace Consistency Contract

Every namespace method chain follows the same 4-step pattern. No exceptions.

```
Namespace . Target . Configuration . Action
```

| Step | What It Does | Examples |
|------|-------------|----------|
| **Namespace** | Which hardware/system | `Screen`, `Sound`, `Input`, `Math` |
| **Target** | What you're operating on | `.Layer(0)`, `.Effect("bounce")`, `.Keyboard` |
| **Configuration** | How to configure it | `.Position(x, y)`, `.Color(r, g, b)`, `.Volume(0.8)` |
| **Action** | Execute (terminal) | `.Draw()`, `.Play()`, `.IsPressed()` |

### Configuration methods are chainable and order-independent

```
// These are equivalent:
Screen.Layer(0).Rect(100, 20).Position(350, 550).Color(255, 255, 255).Draw()
Screen.Layer(0).Rect(100, 20).Color(255, 255, 255).Position(350, 550).Draw()
```

### Terminal actions end the chain

After `.Draw()`, `.Play()`, `.Start()`, `.IsPressed()` — no more chaining. These return a result value (or void), not the builder.

---

## Part 4: The Implicit Runtime Contract

G-Basic eliminates boilerplate by making common setup **implicit**. The runtime handles these automatically:

### What the runtime does for you (zero ceremony)

| Concern | How It's Handled | Beginner Sees |
|---------|-----------------|---------------|
| **Window creation** | First use of `Screen` opens a window (default 800x600) | Nothing — it just works |
| **Frame loop** | `while true { }` at top level is detected as a game loop; runtime wraps with frame timing | Just write `while true { }` |
| **Input polling** | Happens automatically at frame start | Just call `key("left")` |
| **Screen present** | Happens automatically at frame end | Just draw things |
| **Frame rate** | Locked to 60 FPS by default | Smooth animation for free |

### What the user CAN control (opt-in)

```
Screen.size = (1024, 768)       // Override default window size
Screen.title = "My Game"        // Set window title
System.fps = 30                 // Override frame rate
let dt = System.frame_time      // Read delta time for advanced physics
```

### What the user NEVER needs to write

```
// BANNED from examples — these are anti-patterns:
Screen.Init(800, 600)           // NO — implicit
System.FrameBegin()             // NO — implicit
System.FrameEnd()               // NO — implicit
Screen.Present()                // NO — implicit
Input.Poll()                    // NO — implicit
```

**Implementation note:** The compiler detects a top-level `while true { }` and wraps the body with frame begin/end/present calls during IR generation. If no game loop is detected, the program runs as a script (no frame timing needed).

---

## Part 5: The Object Model

Screen objects (shapes, sprites, text) are **persistent handles** with mutable properties. This is the key insight that makes Layer 2 work.

### Creating objects

```
let paddle = rect(100, 20)          // Layer 1 shortcut
let paddle = Screen.Rect(100, 20)   // Layer 2 explicit
```

Both return an **object handle** — an opaque reference to a drawable thing managed by the runtime.

### Auto-draw contract

Objects are **drawn automatically** every frame in layer order. The beginner never calls `.draw()` on persistent objects. `clear(black)` clears the background — it does NOT hide objects. Objects disappear only when `visible = false` or `.remove()` is called.

Draw order: objects on `layer 0` draw first, `layer 1` on top of that, etc. Within the same layer, objects draw in creation order.

### Object properties (read/write)

Every screen object has these standard properties:

**Visual properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `position` | Point | (0, 0) | Top-left position |
| `position.x` | Float | 0 | X coordinate |
| `position.y` | Float | 0 | Y coordinate |
| `size` | Size | (from constructor) | Width and height |
| `size.width` | Float | (from constructor) | Width |
| `size.height` | Float | (from constructor) | Height |
| `color` | Color | white | Fill color |
| `visible` | Bool | true | Whether to draw |
| `layer` | Int | 0 | Drawing layer (higher = on top) |
| `rotation` | Float | 0 | Rotation in degrees |
| `scale` | Float | 1.0 | Scale factor |
| `opacity` | Float | 1.0 | Transparency (0.0-1.0) |

**Physics properties:**

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `velocity` | Point | (0, 0) | Velocity — object moves by this many pixels per frame |
| `velocity.x` | Float | 0 | Horizontal velocity |
| `velocity.y` | Float | 0 | Vertical velocity |
| `gravity` | Float | 0 | Downward acceleration per frame (0 = no gravity) |
| `solid` | Bool | false | Blocks other objects; objects with gravity rest on solid objects |
| `bounces` | Bool | false | Reverses velocity on collision with screen edges or solid objects |

**How the runtime uses physics properties each frame:**

1. `velocity.y = velocity.y + gravity` (gravity accelerates downward)
2. `position = position + velocity` (velocity moves the object)
3. If `bounces` and hits screen edge → reverse the relevant velocity component
4. If `bounces` and hits a `solid` object → bounce off it
5. If `gravity > 0` and resting on a `solid` object → stop falling (velocity.y = 0)
6. If `solid` and hit by a moving object → the moving object's collision is detected

**Why properties instead of a strategy pattern:**

The beginner already knows how properties work from `.position`, `.color`, `.visible`. Physics uses the exact same mental model — set a property, the runtime does the rest. No new concepts, no new namespace, no constructors to remember.

```
// Pong ball — bounces off walls
ball.velocity = (3, 3)
ball.bounces = true

// Flappy bird — falls with gravity
bird.gravity = 0.5

// Angry Birds block — falls and stacks
block.gravity = 0.3
block.solid = true

// Scrolling pipe — constant velocity, no physics
pipe.velocity = (-3, 0)
```

**One pattern, four games worth of physics.**

### Object methods

| Method | Description |
|--------|-------------|
| `object.move(dx, dy)` | Move by offset (one-time push, doesn't change `.velocity`) |
| `object.collides(other)` | Check collision with another object (returns Bool) |
| `object.contains(x, y)` | Check if point is inside (returns Bool) |
| `object.remove()` | Remove from screen and all collections |

### Collision detection (built-in)

```
if ball.collides(paddle) {
    ball.velocity.y = -ball.velocity.y
    score = score + 1
    play("bounce")
}
```

**Rule:** Collision detection is a **first-class feature**, not something beginners must implement with manual coordinate math. The runtime handles AABB (axis-aligned bounding box) collision for all screen objects.

**Collision vs physics:** `.collides()` is a **query** — it tells you IF two objects overlap. The physics engine handles automatic responses for `bounces` and `solid` objects. But the beginner can ALSO check `.collides()` to add custom responses (score, sound, game over).

### Collections (arrays)

Games need multiple objects. G-Basic supports arrays with simple syntax:

```
let blocks = []                     // empty array
blocks.add(block)                   // add an object
blocks.remove(block)                // remove a specific object
blocks.length                       // number of items (property, no parens)

for block in blocks {               // iterate
    if bird.collides(block) {
        play("crash")
    }
}

for i in 0 to 4 {                  // counted loop (0, 1, 2, 3, 4)
    let block = rect(30, 30)
    block.position = Point(500, Screen.height - 70 - i * 30)
    blocks.add(block)
}
```

**Rule:** Arrays are the ONLY collection type. No maps, no sets, no queues. Arrays cover every beginner game pattern (multiple enemies, pipes, blocks, bullets).

---

## Part 6: Sugar Syntax Structure

### The Golden Rule of Sugar

Every sugar form must satisfy ALL of these criteria:

1. **Memorable** — A beginner can recall it after seeing it once
2. **Guessable** — A beginner who hasn't seen it can guess it from context
3. **Chainable** — It returns a builder or object, enabling `.method()` continuation
4. **Transparent** — Intellisense shows what it desugars to
5. **Consistent** — It follows the same pattern as every other sugar form

### Sugar Alias Table (Complete)

#### Output

| Shortcut | Desugars To | Notes |
|----------|-------------|-------|
| `print(args)` | `Screen.Layer(0).Text(args).Draw()` | Default position: next line |
| `print(args).at(x, y)` | `Screen.Layer(0).Text(args).Position(x, y).Draw()` | Positioned text |
| `clear(color)` | `Screen.Layer(0).Clear(color)` | Accepts named color or RGB |
| `clear(r, g, b)` | `Screen.Layer(0).Clear(Color(r, g, b))` | RGB overload |

#### Shapes

| Shortcut | Desugars To | Returns |
|----------|-------------|---------|
| `rect(w, h)` | `Screen.Rect(w, h)` | Object handle |
| `circle(r)` | `Screen.Circle(r)` | Object handle |
| `line(from, to)` | `Screen.Line(from, to)` | Object handle (from/to are Points) |
| `text(content)` | `Screen.Text(content)` | Object handle |
| `sprite(name)` | `Screen.Sprite(name)` | Object handle |

#### Math

| Shortcut | Desugars To |
|----------|-------------|
| `random(min, max)` | `Math.Random(min, max)` |
| `abs(x)` | `Math.Abs(x)` |
| `sqrt(x)` | `Math.Sqrt(x)` |
| `sin(x)` | `Math.Sin(x)` |
| `cos(x)` | `Math.Cos(x)` |
| `clamp(v, lo, hi)` | `Math.Clamp(v, lo, hi)` |
| `round(x)` | `Math.Round(x)` |
| `floor(x)` | `Math.Floor(x)` |
| `ceil(x)` | `Math.Ceil(x)` |
| `min(a, b)` | `Math.Min(a, b)` |
| `max(a, b)` | `Math.Max(a, b)` |

#### System

| Shortcut | Desugars To |
|----------|-------------|
| `wait(secs)` | `System.Wait(secs)` |
| `log(args)` | `System.Log(args)` |

#### Input

| Shortcut | Desugars To |
|----------|-------------|
| `key(name)` | `Input.Keyboard.Key(name).IsPressed()` |
| `mouse.x` | `Input.Mouse.x` (property) |
| `mouse.y` | `Input.Mouse.y` (property) |
| `mouse.clicked` | `Input.Mouse.clicked` (property) |

#### Sound

| Shortcut | Desugars To |
|----------|-------------|
| `play(name)` | `Sound.Effect(name).Play()` |

### Bundled Assets (The Asset Namespace)

G-Basic ships with **bundled assets** so beginners never need to find or download files to make games work. The `play()` and `sprite()` shortcuts use bundled assets by name.

**Bundled sound effects** (used by `play(name)`):

| Name | Sound | Use Case |
|------|-------|----------|
| `"bounce"` | Short click/pop | Ball hitting paddle/wall |
| `"coin"` | Bright chime | Collecting items |
| `"lose"` | Descending tone | Game over / life lost |
| `"win"` | Ascending fanfare | Level complete / victory |
| `"jump"` | Quick whoosh | Character jumping |
| `"flap"` | Soft wing beat | Flappy-style games |
| `"crash"` | Impact thud | Object collision / destruction |
| `"launch"` | Swoosh | Projectile fired |
| `"squeal"` | Cartoon squeak | Enemy defeated |
| `"click"` | UI click | Menu selection |

**Bundled sprites** (used by `sprite(name)`):

| Name | Description |
|------|-------------|
| `"bird"` | Simple bird silhouette |
| `"pig"` | Round pig face |
| `"star"` | Five-pointed star |
| `"heart"` | Heart shape |
| `"arrow"` | Directional arrow |

**Custom assets** (via the `Asset` namespace):

```
// Load a custom image
let hero = sprite("hero")                           // bundled — just works
let custom = Asset.Sprite("my_character.png")       // custom file from project folder

// Load a custom sound
play("bounce")                                       // bundled — just works
let boom = Asset.Sound("explosion.wav")             // custom file
boom.play()                                          // play the custom sound

// Asset search path: project folder → bundled assets → error
```

**Implementation:** `play("bounce")` and `sprite("bird")` first check the project folder for a matching file, then fall back to bundled assets. The `Asset` namespace provides explicit loading with full path control. Bundled assets are embedded in the runtime binary (~50MB total: 10-15 sound effects, 5-10 sprites, all CC0 licensed).

**Rule:** Every example must work with ZERO external files. Bundled assets are the default. Custom assets are opt-in for intermediate users.

---

## Part 7: Relative vs Static Programming

G-Basic supports two complementary styles. Both are valid, both can be mixed freely.

### Static Style (Absolute Coordinates)

```
let paddle = rect(100, 20)
paddle.position = Point(350, 550)
```

Good for: precise layouts, pixel-perfect positioning, learning coordinate systems.

### Relative Style (Semantic Positioning)

```
let paddle = rect(100, 20)
paddle.position = Screen.bottom_center
paddle.position.y = paddle.position.y - 30   // 30px above bottom
```

Good for: responsive layouts, quick prototyping, readable code.

### Relative Operators

```
// Move relative to current position
paddle.move(5, 0)                    // move right 5px
paddle.move(left * 5)               // move left 5px (using direction constant)

// Position relative to screen
paddle.position.x = Screen.width / 2 - paddle.size.width / 2   // center horizontally

// Position relative to another object
ball.position = paddle.position + Point(50, -20)   // above paddle center
```

---

## Part 8: The Week 1 Game Tests

These are the **canonical benchmarks**. If a beginner with intellisense cannot produce something close to these in the stated time, the API has failed.

### Test 1: Pong (10 minutes)

```
// Pong

let paddle = rect(100, 20)
paddle.position = Screen.bottom_center
paddle.position.y = paddle.position.y - 30
paddle.color = white
paddle.solid = true

let ball = circle(10)
ball.position = Screen.center
ball.color = yellow
ball.velocity = (3, 3)
ball.bounces = true

let score = 0

while true {
    if key("left")  { paddle.move(-8, 0) }
    if key("right") { paddle.move(8, 0) }

    // Ball fell off bottom — reset
    if ball.position.y > Screen.height {
        ball.position = Screen.center
        ball.velocity = (3, 3)
        score = 0
        play("lose")
    }

    if ball.collides(paddle) {
        score = score + 1
        play("bounce")
    }

    clear(black)
    print("Score: {score}").at(10, 10)
}
```

**Why this works:** `ball.bounces = true` handles wall AND paddle bouncing automatically (paddle is `solid`). The beginner only writes custom logic for the "fell off bottom" reset and score counting. **35 lines.**

### Test 2: Flappy Bird (15 minutes)

```
// Flappy Bird

let bird = circle(15)
bird.position = Point(100, Screen.center.y)
bird.color = yellow
bird.gravity = 0.5

let ground = rect(Screen.width, 20)
ground.position = Point(0, Screen.height - 20)
ground.color = green
ground.solid = true

let pipes = []
let score = 0
let timer = 0

while true {
    if key("space") {
        bird.velocity.y = -8
        play("flap")
    }

    // Spawn pipes
    timer = timer + 1
    if timer % 90 == 0 {
        let gap_y = random(100, 400)

        let top = rect(60, gap_y)
        top.position = Point(Screen.width, 0)
        top.color = green
        top.velocity.x = -3
        top.solid = true
        pipes.add(top)

        let bot = rect(60, Screen.height - gap_y - 150)
        bot.position = Point(Screen.width, gap_y + 150)
        bot.color = green
        bot.velocity.x = -3
        bot.solid = true
        pipes.add(bot)
    }

    // Collision with pipes or ground
    for pipe in pipes {
        if bird.collides(pipe) {
            play("hit")
            bird.position = Point(100, Screen.center.y)
            bird.velocity.y = 0
            score = 0
        }
        if pipe.position.x < -60 {
            pipe.remove()
            score = score + 1
        }
    }

    if bird.collides(ground) or bird.position.y < 0 {
        play("lose")
        bird.position = Point(100, Screen.center.y)
        bird.velocity.y = 0
    }

    clear(cyan)
    print("Score: {score}").at(10, 10)
}
```

**Why this works:** `bird.gravity = 0.5` makes the bird fall. `bird.velocity.y = -8` is a "flap" impulse. Pipes scroll left via `velocity.x = -3`. Ground is `solid` so the bird rests on it. **~60 lines** — slightly longer but uses the same properties the beginner already knows from Pong.

### Test 3: Angry Birds (20 minutes)

```
// Angry Birds

let ground = rect(Screen.width, 40)
ground.position = Point(0, Screen.height - 40)
ground.color = green
ground.solid = true

let sling = Point(120, Screen.height - 80)

let bird = circle(15)
bird.position = sling
bird.color = red

// Build a tower
let blocks = []
for i in 0 to 4 {
    let block = rect(30, 30)
    block.position = Point(550, Screen.height - 70 - i * 30)
    block.color = brown
    block.gravity = 0.3
    block.solid = true
    blocks.add(block)
}

let pig = circle(18)
pig.position = Point(565, Screen.height - 220)
pig.color = green
pig.gravity = 0.3

let launched = false
let score = 0

while true {
    // Launch bird with space
    if not launched and key("space") {
        bird.velocity = (8, -10)
        bird.gravity = 0.4
        launched = true
        play("launch")
    }

    // Bird hits blocks
    for block in blocks {
        if bird.collides(block) {
            play("crash")
        }
    }

    // Bird or block hits pig
    if bird.collides(pig) or pig.collides(ground) == false {
        pig.remove()
        score = score + 1
        play("squeal")
    }

    // Bird off screen — reset
    if launched and (bird.position.x > Screen.width or bird.position.y > Screen.height) {
        bird.position = sling
        bird.velocity = (0, 0)
        bird.gravity = 0
        launched = false
    }

    clear(cyan)
    print("Score: {score}").at(10, 10)
}
```

**Why this works:** Same four physics properties cover everything:
- Bird: `gravity` + `velocity` = projectile arc
- Blocks: `gravity` + `solid` = stack on ground, get knocked when hit
- Ground: `solid` = everything rests on it
- The `bounces` property from Pong isn't needed here — different game, different physics mix

**~60 lines.** The beginner uses the SAME properties they learned in Pong and Flappy Bird, just combined differently.

### The teaching progression

| Game | Time | New concepts learned | Physics properties used |
|------|------|---------------------|------------------------|
| **Pong** | 10 min | Objects, properties, collision, `velocity`, `bounces`, `solid` | `velocity`, `bounces`, `solid` |
| **Flappy Bird** | 15 min | `gravity`, arrays, `for` loops, spawning objects | `velocity`, `gravity`, `solid` |
| **Angry Birds** | 20 min | Projectile launch, stacking, combining all properties | `velocity`, `gravity`, `solid` |

**All three games use the same 4 physics properties. No new APIs, no new namespaces, no new patterns.**

### What intellisense provides at each step

| User Types | Intellisense Shows |
|------------|-------------------|
| `r` | `rect(width, height)` — Create a rectangle |
| `paddle.` | `position`, `color`, `size`, `velocity`, `gravity`, `solid`, `bounces`, `visible`, `move()`, `collides()`, ... |
| `paddle.position = ` | `Screen.center`, `Screen.top_left`, `Screen.bottom_center`, `Point(x, y)`, ... |
| `paddle.color = ` | `white`, `red`, `blue`, `yellow`, `Color(r, g, b)`, ... |
| `ball.velocity = ` | `(dx, dy)`, `Point(dx, dy)` — Velocity per frame |
| `ball.gravity = ` | `0` (none), `0.3` (gentle), `0.5` (normal), `1.0` (heavy) |
| `ball.bounces = ` | `true`, `false` |
| `ball.solid = ` | `true`, `false` |
| `key(` | `"left"`, `"right"`, `"up"`, `"down"`, `"space"`, ... |
| `ball.c` | `collides(other)` — Check collision with another object |
| `play(` | `"bounce"`, `"coin"`, `"lose"`, `"win"`, `"flap"`, `"crash"`, `"launch"`, ... |
| `clear(` | `black`, `white`, `cyan`, `Color(r, g, b)`, ... |

---

## Part 9: Anti-Patterns (What NOT to Do)

### Anti-Pattern 1: Flat multi-argument calls

```
// BAD — 7 positional arguments, impossible to remember
Screen.DrawRect(paddle_x, paddle_y, paddle_w, paddle_h, 255, 255, 255)

// GOOD — object with named properties
let paddle = rect(100, 20)
paddle.position = Point(350, 550)
paddle.color = white
```

**Rule:** No namespace method should take more than 3 arguments. If it needs more, use an object with properties or a chain with named configuration methods.

### Anti-Pattern 2: Boilerplate ceremony

```
// BAD — beginner must know about frame management
Screen.Init(800, 600)
while true {
    System.FrameBegin()
    // ... game code ...
    Screen.Present()
    System.FrameEnd()
}

// GOOD — runtime handles it
while true {
    // ... game code ...
}
```

**Rule:** If every game program needs the same lines, those lines should be implicit.

### Anti-Pattern 3: Manual collision math

```
// BAD — error-prone, teaches nothing about collision detection
if ball_y > 540.0 and ball_y < 570.0 {
    if ball_x > paddle_x and ball_x < paddle_x + paddle_w {
        ball_vy = 0.0 - ball_vy
    }
}

// GOOD — built-in collision detection
if ball.collides(paddle) {
    ball.velocity.y = -ball.velocity.y
}
```

**Rule:** Common game operations (collision, distance, angle) are built-in methods, not manual math exercises.

### Anti-Pattern 4: Workaround for missing operators

```
// BAD — negation workaround
ball_vx = 0.0 - ball_vx

// GOOD — unary negation
ball.velocity.x = -ball.velocity.x
```

**Rule:** If a beginner would naturally write `-x`, it must work.

### Anti-Pattern 5: Magic numbers without context

```
// BAD — what do these numbers mean?
if paddle_x < 700 {
    paddle_x = paddle_x + 8
}

// GOOD — self-documenting
if paddle.position.x < Screen.width - paddle.size.width {
    paddle.move(8, 0)
}
```

**Rule:** Examples should use `Screen.width`, `Screen.height`, and object properties instead of hardcoded screen dimensions.

### Anti-Pattern 6: Inconsistent API surface

```
// BAD — some methods are flat, some are chained
Screen.DrawRect(x, y, w, h, r, g, b)    // flat
Screen.Layer(1).Sprite("hero").Draw()     // chained

// GOOD — everything follows the same pattern
Screen.Rect(w, h).Position(x, y).Color(r, g, b).Draw()    // chained
Screen.Layer(1).Sprite("hero").Draw()                       // chained
```

**Rule:** The API surface must be **uniform**. No flat utility functions mixed with chained builders.

### Anti-Pattern 7: Separate velocity variables instead of physics properties

```
// BAD — manual velocity tracking, manual movement, manual bounce logic
let ball_vx = 3.0
let ball_vy = 3.0
ball.move(ball_vx, ball_vy)
if ball.position.x < 0 { ball_vx = -ball_vx }

// GOOD — physics properties on the object itself
ball.velocity = (3, 3)
ball.bounces = true
```

**Rule:** Use `.velocity`, `.gravity`, `.solid`, `.bounces` properties to describe physics. The runtime handles the frame-by-frame math. Separate `vx`/`vy` variables are an anti-pattern.

### Anti-Pattern 8: Method parentheses on properties

```
// BAD — Screen.Width() looks like an action, but it's just reading a value
if paddle.position.x < Screen.Width() - paddle.size.width {

// GOOD — properties are properties, no parentheses
if paddle.position.x < Screen.width - paddle.size.width {
```

**Rule:** If it reads or writes a stored value, it's a property (no `()`). If it performs an action or computation, it's a method (with `()`). See the Property Rule table in Part 2.

---

## Part 10: Implementation Priority

### Phase 1 — Must Have (Week 1-2)

These features are required for the Week 1 game tests (Pong, Flappy Bird, Angry Birds):

| Feature | Component | Notes |
|---------|-----------|-------|
| `rect()`, `circle()` constructors | Parser + Runtime | Return object handles |
| Object `.position`, `.color`, `.visible` properties | Runtime | Read/write via handle |
| Physics properties (`.velocity`, `.gravity`, `.solid`, `.bounces`) | Runtime | Built-in physics engine |
| Auto-draw (persistent objects drawn each frame) | Runtime | No manual `.draw()` calls |
| Object `.move()` method | Runtime | One-time offset (doesn't change `.velocity`) |
| Object `.collides()` method | Runtime | AABB collision detection |
| Object `.remove()` method | Runtime | Remove from screen + collections |
| Arrays (`[]`, `.add()`, `.remove()`, `.length`, `for-in`) | Parser + Runtime | Multiple objects |
| `for i in 0 to N` counted loops | Parser | Procedural level building |
| `key(name)` shortcut | Parser (desugar) | Returns bool |
| `print(text).at(x, y)` | Parser (desugar) + Runtime | Positioned text |
| `clear(color)` shortcut | Parser (desugar) | Named colors or RGB |
| `play(name)` shortcut | Parser (desugar) + Runtime | Sound effect playback |
| `random(min, max)` shortcut | Parser (desugar) | Random number generation |
| Named colors (`white`, `black`, `cyan`, etc.) | Runtime globals | Color type |
| Position properties (`Screen.center`, etc.) | Screen namespace | Computed at runtime |
| Implicit frame management | IR generation | Detect `while true` |
| Implicit window creation | Runtime | On first Screen use |
| Unary negation (`-x`) | Parser (already in AST) | Verify it works |
| Modulo operator (`%`) | Parser + Codegen | Needed for spawn timing |
| `not` operator | Parser | Needed for `if not launched` |

### Phase 2 — Should Have (Week 3-4)

| Feature | Component | Notes |
|---------|-----------|-------|
| `mouse.x`, `mouse.y`, `mouse.clicked` | Runtime globals | Mouse input for drag-to-aim |
| `sprite(name)` constructor | Runtime | Load from bundled assets |
| Direction constants (`up`, `down`, etc.) | Runtime globals | Vector2 values |
| `Screen.size`, `Screen.title` properties | Runtime | Override defaults |
| `System.fps`, `System.frame_time` properties | Runtime | Override frame rate, delta time |
| `object.rotation`, `.scale`, `.opacity` | Runtime | Advanced visual properties |
| `object.contains(x, y)` | Runtime | Point-in-object test |

### Phase 3 — Nice to Have (Month 2+)

| Feature | Component | Notes |
|---------|-----------|-------|
| `object.animate()` | Runtime | Sprite animation |
| `object.tween()` | Runtime | Smooth property transitions |
| Particle systems | Runtime | `Screen.Particles()` |
| Camera/viewport | Runtime | `Screen.Camera()` |
| Tilemaps | Runtime | `Screen.Tilemap()` |
| Advanced physics (friction, elasticity, joints) | Runtime | For more complex simulations |

---

## Part 11: Rules for AI Code Generation

When generating G-Basic code (examples, tests, documentation), AI must follow these rules:

### Rule 1: Layer 1 First

Always use the simplest form that works. If `rect(100, 20)` works, don't write `Screen.Rect(100, 20)`. If `key("left")` works, don't write `Input.Keyboard.Key("left").IsPressed()`.

### Rule 2: Objects Over Coordinates

Prefer `let paddle = rect(100, 20)` with `paddle.position = ...` over tracking `paddle_x`, `paddle_y`, `paddle_w`, `paddle_h` as separate variables.

### Rule 3: Named Constants Over Magic Numbers

Use `white` not `255, 255, 255`. Use `Screen.center` not `Point(400, 300)`. Use `Screen.width` not `800`.

### Rule 4: Built-In Methods Over Manual Math

Use `ball.collides(paddle)` not manual AABB checks. Use `paddle.move(8, 0)` not `paddle_x = paddle_x + 8`.

### Rule 5: No Boilerplate

Never include `Screen.Init()`, `System.FrameBegin()`, `System.FrameEnd()`, `Screen.Present()`, or `Input.Poll()` in examples. These are implicit.

### Rule 6: Maximum 3 Arguments

No function or method call should take more than 3 positional arguments. Use object properties or chained configuration methods instead.

### Rule 7: Consistent Patterns

If one example uses `paddle.color = white`, all examples must use the same pattern. No mixing `paddle.color = white` with `Screen.DrawRect(..., 255, 255, 255)`.

### Rule 8: Sound in Every Game Example

Every game example must include at least one `play()` call. The roadmap's success metric is "moving sprite + sound in 15 minutes."

### Rule 9: Score Display in Every Game Example

Every game example must display a score or status using `print("Score: {score}").at(x, y)`. This demonstrates string interpolation and positioned text.

### Rule 10: Comments Explain Why, Not What

```
// BAD
// Move paddle left
paddle.move(-8, 0)

// GOOD
// Arrow keys control the paddle
if key("left") { paddle.move(-8, 0) }
```

---

## Part 12: Reconciliation with Current Implementation

The current codebase (as of 2026-02-13) does NOT support all features in this document. Here is the gap:

### What exists today

- Namespace method chains (`Screen.DrawRect(...)`) — parser + codegen
- Flat namespace calls (no object model) — runtime
- `System.FrameBegin()` / `System.FrameEnd()` — runtime
- `Screen.Init()` / `Screen.Present()` — runtime
- `Input.KeyPressed()` — runtime (flat, not chained)
- Basic math functions — runtime

### What must be built to match these guardrails

| Gap | Priority | Effort | Notes |
|-----|----------|--------|-------|
| Object handle system (rect/circle return handles) | Critical | 8h | Runtime: handle table + property storage |
| Object properties (`.position`, `.color`, `.visible`) | Critical | 6h | Parser: field access on handles; Runtime: property get/set |
| Physics properties (`.velocity`, `.gravity`, `.solid`, `.bounces`) | Critical | 10h | Runtime: per-frame physics step |
| Auto-draw (persistent objects drawn each frame) | Critical | 4h | Runtime: iterate handle table, draw visible objects |
| `.collides()` method | Critical | 4h | Runtime: AABB check between two handles |
| `.move()` method | Critical | 2h | Runtime: offset position by delta |
| `.remove()` method | Critical | 2h | Runtime: remove from handle table + arrays |
| Arrays (`[]`, `.add()`, `.remove()`, `for-in`) | Critical | 8h | Parser: array literal, methods; Runtime: dynamic array |
| `for i in 0 to N` counted loops | Critical | 3h | Parser: range expression + for loop |
| Named color constants | High | 2h | Runtime: global Color values |
| Position properties (`Screen.center`, etc.) | High | 3h | Screen namespace properties |
| Implicit frame management | High | 4h | IR gen: detect `while true`, wrap body |
| Implicit window creation | High | 2h | Runtime: lazy init on first Screen use |
| `print().at()` chain | High | 3h | Parser desugar + Runtime text rendering |
| Shape constructor shortcuts (`rect()`, `circle()`) | High | 2h | Parser: alias table entries |
| `random(min, max)` | High | 1h | Parser desugar + Runtime |
| Modulo operator (`%`) | High | 1h | Parser + Codegen |
| `not` operator | High | 1h | Parser + Codegen |
| Remove `Screen.Init` / `FrameBegin` / `FrameEnd` requirement | High | 2h | Runtime refactor |

### Migration strategy

1. **Keep existing flat API working** — don't break current examples
2. **Add object model alongside** — new API coexists with old
3. **Update examples to new style** — once object model works
4. **Deprecate flat API** — after all examples are migrated
5. **Remove flat API** — in v1.0 release

---

## Appendix A: Pong — All Three Layers

### Layer 1 (Beginner — 10 minutes)

Physics properties handle everything. Beginner focuses on game logic.

```
let paddle = rect(100, 20)
paddle.position = Screen.bottom_center
paddle.position.y = paddle.position.y - 30
paddle.color = white
paddle.solid = true

let ball = circle(10)
ball.position = Screen.center
ball.color = yellow
ball.velocity = (3, 3)
ball.bounces = true

let score = 0

while true {
    if key("left")  { paddle.move(-8, 0) }
    if key("right") { paddle.move(8, 0) }

    if ball.position.y > Screen.height {
        ball.position = Screen.center
        ball.velocity = (3, 3)
        score = 0
        play("lose")
    }

    if ball.collides(paddle) {
        score = score + 1
        play("bounce")
    }

    clear(black)
    print("Score: {score}").at(10, 10)
}
```

### Layer 2 (Intermediate — explicit objects, manual wall bouncing)

Uses `.velocity` but handles wall bouncing manually. No `bounces = true`.

```
let paddle = Screen.Rect(100, 20)
paddle.position = Point(350, 550)
paddle.color = Color(255, 255, 255)
paddle.solid = true

let ball = Screen.Circle(10)
ball.position = Point(400, 300)
ball.color = Color(255, 255, 0)
ball.velocity = (3, 3)

let score = 0

while true {
    if Input.Keyboard.Key("left").IsPressed() and paddle.position.x > 0 {
        paddle.move(-8, 0)
    }
    if Input.Keyboard.Key("right").IsPressed() and paddle.position.x < Screen.width - paddle.size.width {
        paddle.move(8, 0)
    }

    // Manual wall bouncing — beginner learns coordinate math
    if ball.position.x < 0 or ball.position.x > Screen.width - ball.size.width {
        ball.velocity.x = -ball.velocity.x
    }
    if ball.position.y < 0 {
        ball.velocity.y = -ball.velocity.y
    }

    if ball.position.y > Screen.height {
        ball.position = Point(400, 300)
        ball.velocity = (3, 3)
        score = 0
        Sound.Effect("lose").Play()
    }

    if ball.collides(paddle) {
        ball.velocity.y = -ball.velocity.y
        score = score + 1
        Sound.Effect("bounce").Play()
    }

    Screen.Layer(0).Clear(Color(0, 0, 0))
    Screen.Layer(0).Text("Score: {score}").Position(10, 10).Draw()
}
```

### Layer 3 (Expert — fully manual, no physics properties)

No `.velocity` at all. Expert manages velocity variables and calls `.move()` directly.

```
let paddle = Screen.Layer(0).Rect(100, 20).Position(350, 550).Color(255, 255, 255).Create()
let ball = Screen.Layer(0).Circle(10).Position(400, 300).Color(255, 255, 0).Create()

let ball_vx = 3.0
let ball_vy = 3.0
let score = 0

while true {
    if Input.Keyboard.Key("left").IsPressed() and paddle.position.x > 0 {
        paddle.move(-8, 0)
    }
    if Input.Keyboard.Key("right").IsPressed() and paddle.position.x < Screen.width - paddle.size.width {
        paddle.move(8, 0)
    }

    ball.move(ball_vx, ball_vy)

    if ball.position.x < 0 or ball.position.x > Screen.width - ball.size.width {
        ball_vx = -ball_vx
    }
    if ball.position.y < 0 {
        ball_vy = -ball_vy
    }

    if ball.position.y > Screen.height {
        ball.position = Point(400, 300)
        ball_vx = 3.0
        ball_vy = 3.0
        score = 0
        Sound.Effect("lose").Play()
    }

    if ball.collides(paddle) {
        ball_vy = -ball_vy
        score = score + 1
        Sound.Effect("bounce").Play()
    }

    Screen.Layer(0).Clear(0, 0, 0)
    Screen.Layer(0).Text("Score: {score}").Position(10, 10).Color(255, 255, 255).Draw()
}
```

---

## Appendix B: Intellisense Discovery Flow (Pong)

This is the exact sequence a beginner follows to build Pong, guided by intellisense:

```
Step 1:  User types "let paddle = "
         Intellisense suggests: rect(), circle(), line(), text(), sprite()
         User picks: rect(

Step 2:  Intellisense shows: rect(width: Int, height: Int) — Create a rectangle
         User types: rect(100, 20)

Step 3:  User types "paddle."
         Intellisense suggests: position, color, size, velocity, gravity, solid, bounces, visible, move(), collides()
         User picks: position

Step 4:  User types "paddle.position = "
         Intellisense suggests: Screen.center, Screen.top_left, Screen.bottom_center, Point(x, y)
         User picks: Screen.bottom_center

Step 5:  User types "paddle.color = "
         Intellisense suggests: white, red, blue, yellow, black, Color(r, g, b)
         User picks: white

Step 6:  User types "paddle.solid = "
         Intellisense suggests: true, false
         User picks: true
         (Tooltip: "Solid objects block other objects and can be bounced off")

Step 7:  User types "let ball = circle(10)"
         (Same pattern — already learned from paddle)

Step 8:  User types "ball.velocity = "
         Intellisense suggests: (dx, dy), Point(dx, dy)
         User types: (3, 3)
         (Tooltip: "Velocity — object moves by this many pixels per frame")

Step 9:  User types "ball.bounces = true"
         (Tooltip: "Reverses velocity on collision with screen edges or solid objects")

Step 10: User types "while true {"
         (Intellisense shows: Game loop — code inside runs 60 times per second)

Step 11: User types "if key("
         Intellisense suggests: "left", "right", "up", "down", "space"
         User picks: "left"

Step 12: User types "ball.collides("
         Intellisense shows: collides(other: Object) — Check collision
         User types: paddle

Step 13: User types "play("
         Intellisense suggests: "bounce", "coin", "lose", "win", "flap", "crash"
         User picks: "bounce"

Step 14: User types "clear("
         Intellisense suggests: black, white, cyan, Color(r, g, b)
         User picks: black

Step 15: User types "print("
         Intellisense shows: print(text: String) — Display text
         User types: "Score: {score}"

Step 16: User types ").at("
         Intellisense shows: at(x: Int, y: Int) — Position on screen
         User types: 10, 10
```

**Total: 16 steps. Each step is one intellisense interaction. Target time: <1 minute per step.**

---

## Appendix C: Checklist for Reviewing Examples

Before any `.gb` example is committed, verify:

- [ ] Uses Layer 1 shortcuts where possible (`rect()`, `key()`, `print()`, `play()`, `clear()`)
- [ ] Uses named colors instead of RGB tuples
- [ ] Uses `Screen.width` / `Screen.height` (properties, no parens) instead of hardcoded dimensions
- [ ] No `Screen.Init()`, `System.FrameBegin()`, `System.FrameEnd()`, `Screen.Present()`
- [ ] No function takes more than 3 positional arguments
- [ ] Game examples include sound (`play()`)
- [ ] Game examples display score/status (`print().at()`)
- [ ] Uses object properties (`.position`, `.color`) instead of separate x/y/w/h variables
- [ ] Uses physics properties (`.velocity`, `.gravity`, `.solid`, `.bounces`) instead of manual velocity loops
- [ ] Uses `.collides()` instead of manual coordinate math (where applicable)
- [ ] Properties use no parens (`Screen.width`), methods use parens (`paddle.move()`)
- [ ] Value types use constructors (`Point(x, y)`, `Color(r, g, b)`) or named constants (`white`, `Screen.center`)
- [ ] Uses unary negation (`-x`) instead of `0 - x`
- [ ] Comments explain intent, not mechanics
- [ ] Total line count is under 65 lines for simple games
