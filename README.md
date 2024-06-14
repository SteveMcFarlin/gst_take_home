
This was a take home project. Scrubbed history to remove any references to the company.

This is a simple camera recorder that exposes a HTTP REST API as the control surface.

### Development

**System Requirments**

- Linux (Tested on Pop!_OS 22.04 LTS)
- [Rust](https://www.rust-lang.org/tools/install) 1.7.0 or later
- gstreamer 1.22.0 or later


**Build, Test, Run**

```
cargo build --bin recorder
``` 
```
cargo test --package recorder -- --nocapture
```
```
cargo run -- <command line options>
```
```
USAGE:
    recorder 0.1.0

USAGE:
    recorder --host <HOST> --port <PORT>

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --host <HOST>    Sets the host to connect to
    -p, --port <PORT>    Sets the port to connect to
```

### API

All API calls use the `Content-Type: application/json` header.


<!-- **Pause Recording**

```
POST http://.../pause
``` -->


**Start Recording**

This is a blocking call. It will return when the GST pipeline has entered into PLAYING state, or if an error occurs.

Returns 
- 200 OK if successful
- 400 Bad Request if the request failed.

```
POST http://.../start
{
    "duration": <unsigned int>, // Optional. Default: infinite
    "input": { // Optional.
        "name": <string>,
        "variant": <object> // Optional. Default: "v4l2"
    },
    "encoder": { // Optional.
        "name": <string>,
        "variant": <object> // Optional. Default: "X265"
    },
    "output": {
        "name": <string>,
        "variant": <object> // Optional. Default: "filesink"
    }
}
```

**Example**

```
{
    "name": "Recordcer",
    "duration": 5,
    "input": {
        "name": "V4L2",
        "variant": {
            "V4l2": {
                "device": "/dev/video4"
            }
        }
    },
    "encoder": {
        "name": "X265",
        "variant": {
            "X265": {
                "bitrate": 5000
            }
        }
    },
    "output": {
        "name": "FileSink",
        "variant": {
            "FileSink": {
                "muxer_config": {
                    "Matroska": {

                    }
                },
                "location": "/tmp/timed.mkv"
            }
        }
    }
}
```

**Stop Recording**

This is a blocking call. It will return when the GST pipeline has processed the EOS event.

Returns 
- 200 OK if successful
- 400 Bad Request if the request failed.

```
POST http://.../stop
```


**Input Configurations**

- V4L2
```
{
    "V4l2": {
        "device": <string>, // Default: "/dev/video0"
    }
    
}
```

- Test Source
```
{
    "Test" {
        "pattern": <string>, // ["smpte", "snow", "black", "white", "red", "green", "blue", "checkers1", "checkers2", "checkers4", "checkers8", "circle", "smpte75", "zoneplate", "gamut", "chroma-zoneplate", "solid-color", "ball", "smpte100", "bar", "pinwheel", "spokes", "gradient", "colors", "bar", "pinwheel", "spokes", "gradient", "colors"]. Default: "smpte"
    }
}
```

**Encoder Configurations**
```
{
    "X264": {
        "bitrate": <unsigned int>, // Default: 1000000
        ... // See src/encoder/x264enc.rs Config for more options
    }
    // OR
    "X265": {
        "bitrate": <unsigned int>, // Default: 1000000
        ... // See src/encoder/x265enc.rs Config for more options
    }
}
```

**Output Configurations**
- File
```
{
    "FileSink": {
        "location": <string>, // Default: "/tmp/output.mp4"
        ... // See src/output/filesink.rs Config for more options
    }
}
```

- FakeSink
```
{
    "FakeSink": {
        ... // See src/output/fakesink.rs Config for options
    }
}
```


**Stop Recording**

```
POST http://.../stop
```


**POST** messages have the following 200 OK responses:

```
{
    "status": "OK",
}
```

Error responses will have the following format:

```
{
    "state": "Stopped | Playing | Error", // Not Implemented
    "error_message": String, 
    "error_pipeline_graph": String, //graphviz dot format - Not Implemented
}
```

### Project Notes

- Given the limited time, I have chosen to make a _statically_ linked pipeline. This is a tradeoff between simplicity and flexibility.
- Personally I would have preferred to use `/config` as an endpoint to configure the pipeline. Reasoning being it is helpful to get errors early while allocation (RAII) is occurring.
- Not all encoders are implemented. Currently only `X264` and `X265` are implemented. 
- Muxer properties are not implemented. Currently just the defaults are used.
- I have taken a slight liberty with some of the code, and used structures from a prior personal project to aid in the speed of development.
- This is a very basic implementation. It is not production ready. It is more along the lines of a proof of concept. For a production system I would focus on architecture and through tests. Error handling would be more robust, and the API would be more flexible.
- There is code smell with respect to configuration of the Encoder. This should be improved. 
- There are most certainly bugs in the code (logic and maybe leaks due to reference counting). I have not had time to really test it.

### License
[Licensed](LICENSE) under LGPL 2.1.

