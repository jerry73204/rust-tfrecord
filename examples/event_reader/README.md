# Event reader example: Extracting training curves from the TensorBoard log

For the common training in deep learning, we usually save the training log
into [TensorBoard](https://github.com/tensorflow/tensorboard) for better
visualization.

For example, the code `./generate_tensorboard.py` adopted from
[PyTorch summary writer tutorial](https://pytorch.org/docs/stable/tensorboard.html)
can generate the following sample TensorBoard log.

```bash
python generate_tensorboard.py
tensorboard --logdir logs
```

![demo](./demo.png)

On the other hand, we may want to extract the data inside the
TensorBoard log, which is just a kind of **TFRecord Event**,
to analyze training behavior.

With the help of rust-tfrecord, we can run this example to extract the
data we want. First, specify the *EVENT_FILE* inside the log folder *logs*,
and run this example to see the tags inside it.

```bash
cargo run \
    --example event_reader \
    --features with-serde \
    -- \
    --event EVENT_FILE
```

*Example output*

```bash
The tags inside this event are [
    "Accuracy/test",
    "Accuracy/train",
    "Loss/test",
    "Loss/train",
]
Please specify the tags to be extracted.
```

Then we can run the example again to conditionally export the
learning curves to CSV files by specifying the tags.

```bash
cargo run --example event_reader \
    --features with-serde \
    -- \
    --event EVENT_FILE \
    --tags Accuracy/test Accuracy/train
```

Example CSV files saved in the output directory.

*output/Accuracy-test.csv*


```csv
step,value
0,0.11688784
1,0.5557087
2,0.59206223
...
```

*output/Accuracy-train.csv*

```csv
step,value
0,0.9095652
1,0.8410302
2,0.13530093
...
```
