## Contract discovery
Inspect the available command surface before writing new automations.

```bash
fpt capabilities --output json
fpt inspect command entity.find --output json
fpt inspect command entity.find-one --output json
```

## Read patterns

### Read by id
Use `entity.get` when the id is already known.

```bash
fpt entity get Shot 123 --output json
```

### Read one matching record
Use `entity.find-one` when the task needs only one result.

```bash
fpt entity find-one Shot --input @query.json --output json
```

`entity.find-one` reuses the same input model as `entity.find`, but returns the first matching entity or `null`.

### Read multiple matching records
Use `entity.find` for collection queries.

Simple query JSON:
```json
{
  "fields": ["code", "sg_status_list"],
  "page": { "size": 20 },
  "filters": {
    "project": "123"
  }
}
```

Structured `_search` JSON:
```json
{
  "fields": ["code", "sg_status_list"],
  "search": {
    "filters": {
      "filter_operator": "all",
      "filters": [
        ["sg_status_list", "is", "ip"]
      ]
    },
    "additional_filter_presets": [
      {
        "preset_name": "LATEST",
        "latest_by": "ENTITIES_CREATED_AT"
      }
    ]
  }
}
```

Filter DSL example:
```bash
fpt entity find Asset --filter-dsl "sg_status_list == 'ip' and (code ~ 'bunny' or id > 100)" --output json
```

## Batch patterns
Use `entity.batch.*` when the task repeats the same command over many items.

```bash
fpt entity batch get Shot --input '{"ids":[101,102],"fields":["code"]}' --output json
```

## Write safety
Use dry-run before create, update, or delete.

```bash
fpt entity create Version --input @payload.json --dry-run --output json
fpt entity update Task 42 --input @patch.json --dry-run --output json
fpt entity delete Playlist 99 --dry-run --output json
```

Delete requires explicit confirmation for real execution:

```bash
fpt entity delete Playlist 99 --yes --output json
```
