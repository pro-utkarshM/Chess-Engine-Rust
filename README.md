# Bringing Chess to Life- Chess Engine

Combining AI's computational power with the classic beauty of a physical chess board in a seamless manner.

## Why write a Chess engine?

I've always looked for methods to close the gap between the traditional chess experience and the digital world because I'm a passionate chess player and tech enthusiast. Even though there are a lot of great online chess games available, such as [lichess](https://lichess.org/) and [chess.com](https://chess.com/), I've never felt the same sense of connection to the real tactile chess game.

I started a project to combine a strong chess engine with a hardware chess board in order to solve this. My goal was to develop a chess engine that could easily interface with any hardware configuration, bringing the excitement of playing against a highly intelligent artificial intelligence to a physical chess board, while utilising the flexibility and efficiency of the Rust programming language.

## How does it work?

This particular AI works using the [Minimax algorithm](https://en.wikipedia.org/wiki/Minimax), along with [Alpha-Beta pruning](https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning) for optimization.

Now, let's unpack that.

The Minimax algorithm essentially iterates through all possible moves recursively, and assumes that whenever the computer plays, the human player will always respond with the best move.

![Move generation](/assets/move-generation.png)

This allows the computer to almost always play objectively better moves than the player.

![Minimax](/assets/mini-max.jpeg)

As you can see with a little experimentation, it works quite well. 


### Abusing Minimax

Because Minimax works by simply maximizing the AI's material advantage over the player, it's incredibly simple to abuse the algorithm by changing what it is maximizing.

Here, for example, is the **_opposite_** of a good AI. This AI tries to maximize _**YOUR**_ material value, and will desperately try to offer you its pieces while still making legal moves.


## Integration with Hardware Chess Board

Now, let's talk about the exciting part – integrating this powerful chess engine with a hardware chess board. Imagine being able to play against a formidable AI opponent while physically interacting with a traditional chess board.

Through the use of sensors and actuators, the hardware chess board can communicate with the chess engine, translating physical moves into digital commands and vice versa. This seamless integration brings the best of both worlds, combining the tangible experience of moving chess pieces with the strategic depth of a sophisticated AI opponent.

