# StreaMeme Backend

This is the backend for StreaMeme web service.

## Usage

First, please ensure you have installed [Rust](https://www.rust-lang.org/). You can follow the instruction [here](https://www.rust-lang.org/tools/install) to install it.

Then, you need to ensure the StreaMeme Inference project is at the location `../streameme_inference` (This is subject to change).

After that, you can invoke the backend with the following command:
```bash
cargo run --release
```
which will start the backend at port 9090.

## APIs

Currently, we only provide a single API `POST /upload`. 

### POST /upload

This API receives `multipart/form-data` requests, which should contain two fields:

- `metadata`: an `application/json` part within schema:
    ```
    {
        "mode": 1
    }
    ```
    "mode" should be either 0 (binary) or 1 (multi). However, binary mode is still not supported at the time of writing.

- `file`: the file part, which should contains the video file to be analyzed.

This API can be tested with `curl`:
```
curl -v -F 'metadata={"mode":1};type=application/json' -F file@<video_file> http://<host>:9090/upload
```
where \<video_file\> is the path to the video you want to analyze, and \<host\> is the host the backend running on.

The API returns responses in the form like this:
```
{
    "file_name": "video.mp4",
    "analyze_time": "2025-09-22 00:21:22.626625114 +00:00:00", 
    "analyze_mode": "multi",
    "suggestions": [
        {
            "start": 30,
            "end": 60,
            "suggestion": "sorrow"
        },
        {
            "start": 300,
            "end": 330,
            "suggestion": "anger"
        }
    ]
}
```
Note that `suggestions` field can be `null`, indicating that the inference process crashed. Such situation is considered as a bug, so please contact us if you encoutered that situation.