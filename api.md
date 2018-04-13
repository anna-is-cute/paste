# API

## POST `/pastes`

Query params:

- `full` (`bool`): `true` or `false` (default: `false`)

  Includes files and their contents if `true`
- `api_key` (`str`): api key for creating private pastes (default: not specified)

Accepts:

```javascript
{
  // name of the paste
  "name": "my files",
  // the visibility of the paste
  // can be any one of the following (* is default)
  // public - publicly available and not hidden from crawlers
  // *unlisted - publicly available to anyone with the link, hidden from crawlers
  // private - only visible to the authed user creating the paste
  "visibility": "public",
  // array of files to add to the paste
  // all files must have content (field content) in a given format (field format)
  // formats are listed below
  // text - valid utf-8 text
  // base64 - base64 of the uncompressed content
  // gzip - base64 of the gzip-compressed content
  // xz - base64 of the xz-compressed content
  "files": [
    {
      // name of the file
      // if not specified, pastefile1, pastefile2, etc.
      "name": "file_1.txt",
      // specify that the content field is valid utf-8 text
      "format": "text",
      // content of the file as valid utf-8 text
      "content": "Hello!"
    },
    {
      "name": "file_2.jpg",
      // specify that the content field is base64-encoded data
      "format": "base64",
      // content of the jpg in base64 (truncated here)
      "content": "/9j/4AAQSkZJRgABAQAAAQABAAD//gA7..."
    }
  ]
}
```

Output (success):

```javascript
{
  // status of creating the paste
  // always one of success or error (tagged enum)
  "status": "success",
  // id of the created paste
  "id": "abcdef1234"
  // TODO: include urls?
}
```

Output (error):

```javascript
{
  // status of creating the paste
  // always one of success or error (tagged enum)
  "status": "error",
  // error code
  "code": 1,
  // error message
  "message": "oh noes"
}
```

## DELETE `/pastes/<id>`

## PATCH `/pastes/<id>`

## GET `/pastes/<id>`

## GET `/pastes/<id>/files`

## GET `/pastes/<id>/files/<id>`
