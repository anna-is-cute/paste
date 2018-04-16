# API

## Base URL

This document assumes a base URL of `/api`.

## POST `/pastes`

Create a new paste.

### Headers

- `Content-Type` (required): `application/json`
- `Authorization` (optional): `Key <api_key>`

  Providing the Authorization header will create a paste using your account. To make an anonymous
  paste, do not include this header.

  If this header is not included, a deletion key will be generated and returned in order to delete
  anonymous pastes. Provide this key instead of an API key to the delete method.

### Accepts

```javascript
{
  // (optional) name of the paste
  "name": "my files",
  // (optional) the visibility of the paste
  // can be any one of the following (* is default)
  // public - publicly available and not hidden from crawlers
  // *unlisted - publicly available to anyone with the link, hidden from crawlers
  // private - only visible to the authed user creating the paste
  "visibility": "public",
  // (required – at least one file) array of files to add to the paste
  "files": [
    {
      // (optional) name of the file
      // if not specified, pastefile1, pastefile2, etc.
      "name": "file_1.txt",
      // (required) the content of the file
      // all files must have content in a given format
      "content": {
        // formats are listed below
        // text - valid utf-8 text
        // base64 - base64 of the uncompressed content
        // gzip - base64 of the gzip-compressed content
        // xz - base64 of the xz-compressed content

        // (required)  the format of the file
        // specify that the content field is valid utf-8 text
        "format": "text",
        // (required)  the value of the file contents
        // content of the file as valid utf-8 text
        "value": "Hello!"
      }
    },
    {
      "name": "file_2.jpg",
      "content": {
        // specify that the content field is base64-encoded data
        "format": "base64",
        // content of the jpg in base64 (truncated here)
        "content": "/9j/4AAQSkZJRgABAQAAAQABAAD//gA7..."
      }
    }
  ]
}
```

### Output (success, `200`)

```javascript
{
  // status of creating the paste
  // always one of success or error (tagged enum)
  "status": "success",
  "result": {
    // id of the created paste
    "id": "abcdef1234",
    // (optional) key to use when deleting this paste, if this paste was made anonymously
    "deletion_key": "ghijkl5678"
    // TODO: include urls?
  }
}
```

### Output (error, `400 | 403 | 404`)

```javascript
{
  // status of creating the paste
  // always one of success or error (tagged enum)
  "status": "error",
  // error key
  "error": "invalid_json",
  // (optional) error message
  "message": "oh noes"
}
```

## DELETE `/pastes/<id>`

Deletes an existing paste.

### Headers

- `Authorization` (required): `Key <api_key/deletion_key>`

  The key used must be linked to the account that owns the paste.

  If the paste was anonymous, use its deletion key (returned when creating the paste) instead of an
  API key.

### Output (success, `204`)

No content

### Output (error, `400 | 403 | 404`)

Standard error (see POST `/pastes`)

## PATCH `/pastes/<id>`

Update an existing paste.

### Headers

- `Content-Type` (required): `application/json`
- `Authorization` (required): `Key <api_key>`

  The API key must be linked to the user that created the paste.

  Anonymous pastes cannot be update.

### Accepts

The same object as POST `/pastes`, but all fields are optional.

Omit a field to leave it untouched, set a field to null to unset it, and set a field to any other
value to update it.

Fields that can be unset:

- name
- files.name

### Output (success, `204`)

No content

### Output (error, `400 | 403 | 404`)

Standard error (see POST `/pastes`)

## GET `/pastes/<id>`

Get an existing paste.

### Query params

- `full` (`bool`): `true` or `false` (default: `false`)

  Includes the contents of files if `true`.

### Headers

- `Authorization` (optional): `Key <api_key>`

  An API key is only necessary when viewing a private paste. The key must be linked to the account
  that created the private paste.

### Output (success, `200`)

```javascript
{
  "status": "success",
  "result": {
    "id": "abc123",
    "name": "my files",
    "visibility": "public",
    "files": [
      {
        "id": "def456",
        "name": "file_1.txt",
        // only included if the query param `full` is `true`
        "content": {
          "format": "text",
          "value": "Hello!"
        }
      },
      {
        "id": "ghi789",
        "name": "file_2.jpg",
        // only included if the query param `full` is `true`
        "content": {
          "format": "base64",
          "content": "/9j/4AAQSkZJRgABAQAAAQABAAD//gA7..."
        }
      }
    ]
  }
}
```

### Output (error, `400 | 403 | 404`)

Standard error (see POST `/pastes`)

## GET `/pastes/<id>/files`

Get an existing paste's files.

### Headers

- `Authorization` (optional): `Key <api_key>`

  An API key is only necessary when viewing a private paste. The key must be linked to the account
  that created the private paste.

### Output (success, `200`)

```javascript
{
  "status": "success",
  "result": [
    {
      "id": "def456",
      "name": "file_1.txt"
    },
    {
      "id": "ghi789",
      "name": "file_2.jpg"
    }
  ]
}
```

### Output (error, `400 | 403 | 404`)

Standard error (see POST `/pastes`)

## GET `/pastes/<id>/files/<id>`

Get one file from an existing paste.

### Headers

- `Authorization` (optional): `Key <api_key>`

  An API key is only necessary when viewing a private paste. The key must be linked to the account
  that created the private paste.

### Output (success, `200`)

```javascript
{
  "status": "success",
  "result": {
    "id": "def456",
    "name": "file_1.txt",
    "content": {
      "format": "text",
      "value": "Hello!"
    }
  }
}
```

### Output (error, `400 | 403 | 404`)

Standard error (see POST `/pastes`)
