# chef-rs
An implementation of the esoteric language Chef in Rust.

## Language

This implementation tries to follow [the specification](https://www.dangermouse.net/esoteric/chef.html) as close as possible.

### Limitations

Due to the way the parser is implemented, it is quite complicated to differentiate between a loop begin and an auxiliary recipe.
That means that often auxiliary recipes need some clear delimiter above them.
Choices are:
- Using a standard `Serves` statement.
- Using a non-standard break line. These are defined as three or more dashes, underscores or equal signs and mark the end of a recipe.

### `Take`ing liquid ingredients

This implementation extends the `Take` syntax:

`Take`ing an ingredient from a refrigerator will now read a single Unicode character code if the ingredient is liquid.

### `Check`ing the refrigerator

There is a new syntax that allows you to check ahead a bit in the input buffer:

`Check the refrigerator for <ingredient name>.`

- If the ingredient is dry, it will be checked if the next character in the input buffer belongs to *a number*. When the buffer is empty, the user will be prompted.
- If the ingredient is liquid, if there is *anything* left in the input buffer. The user will not be prompted.

Depending on the outcome, the ingredient will either contain 1.0 or 0.0.

### `Examine`ing stuff

You can `Examine` ingredients or mixing bowls, which will display their current status.

This is intended for debugging.
