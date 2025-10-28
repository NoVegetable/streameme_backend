# StreaMeme Backend

This is the backend for StreaMeme web service.

## Usage

First, please ensure you have installed [Rust](https://www.rust-lang.org/). You can follow the instruction [here](https://www.rust-lang.org/tools/install) to install it.

Then, please check whether StreaMeme Inference project is at the location `../streameme_inference`, since this the default location that the backend looks for.
If it is not there, you can still configure it through `inference_dir` option.

After that, you can invoke the backend with the following command:
```bash
cargo run --release
```
which will start the backend at port 9090. You can also choose another port using `--port` (or `-p`) option, such as
```bash
cargo run -- --port 6789
```
As mentioned above, you can configure the location of StreaMeme Inference project using `--inference_dir` option, such as
```bash
cargo run -- --inference_dir "$HOME"/streameme_inference
```
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
    "mode" should be either 0 (binary) or 1 (multi). However, binary mode is still not supported at the time of writing, thus setting `mode` to 0 still invoke the same inference procedure as setting it to 1.

- `file`: the file part, which should contains the video file to be analyzed.
  - Currently, the size limit for the video is set to **2 GiB**. The backend will return a "Payload error" message for any video beyonds this limit.

This API can be tested with `curl`:
```
curl -v -F 'metadata={"mode":1};type=application/json' -F file=@<video_file> http://<host>:<port>/upload
```
where `<video_file>` is the path to the video you want to analyze, `<host>` is the host the backend running on, and `<port>` is the port number the backend is listening on.

The API returns responses in the form like this:
```
{
    "file_name": "video.mp4",
    "analyze_time": "2025-09-22T00:21:22.626Z", 
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
