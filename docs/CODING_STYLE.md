# CODING_STYLE.md

This document defines strict, example-driven coding rules.

Each rule shows BAD and GOOD patterns.

Use the GOOD patterns.

Avoid narrative comments.

Avoid cleverness.

Avoid hidden state.

Avoid unnecessary abstraction.

Prefer code that plainly identifies what it does.

=====================================================================
1. AVOID DEEP NESTING
=====================================================================

Python BAD

def authorize(user):
    if user:
        if user.is_active:
            if user.role == "admin":
                return True
    return False

Python GOOD

def authorize(user):
    if not user:
        return False
    if not user.is_active:
        return False
    if user.role != "admin":
        return False
    return True


Go BAD

if user != nil {
    if user.Active {
        if user.CanWrite {
            return save(user)
        }
    }
}
return errors.New("invalid user")

Go GOOD

if user == nil {
    return errors.New("nil user")
}
if !user.Active {
    return errors.New("inactive user")
}
if !user.CanWrite {
    return errors.New("write permission required")
}
return save(user)


Rust BAD

match user {
    Some(user) => match user.active {
        true => match user.can_write {
            true => save(user),
            false => Err(UserError::WritePermissionRequired),
        },
        false => Err(UserError::Inactive),
    },
    None => Err(UserError::Missing),
}

Rust GOOD

let user = user.ok_or(UserError::Missing)?;

if !user.active {
    return Err(UserError::Inactive);
}

if !user.can_write {
    return Err(UserError::WritePermissionRequired);
}

save(user)


TypeScript BAD

if (user) {
  if (user.active) {
    if (user.role === "admin") {
      return true
    }
  }
}
return false

TypeScript GOOD

if (!user) return false
if (!user.active) return false
return user.role === "admin"


Pattern

Fail early.

Return early.

Keep the primary path straight.


=====================================================================
2. TOP-DOWN NARRATIVE ORDER
=====================================================================

Python BAD

def validate_user(user):
    raise NotImplementedError

def write_audit_record(user):
    raise NotImplementedError

def process_user(user):
    validate_user(user)
    write_audit_record(user)
    return compute_user_result(user)

Python GOOD

def process_user(user):
    validate_user(user)
    write_audit_record(user)
    return compute_user_result(user)

def validate_user(user):
    raise NotImplementedError

def write_audit_record(user):
    raise NotImplementedError

def compute_user_result(user):
    raise NotImplementedError


Go BAD

func validateUser(user User) error {
    panic("example body omitted")
}

func writeAuditRecord(user User) error {
    panic("example body omitted")
}

func ProcessUser(user User) error {
    if err := validateUser(user); err != nil {
        return err
    }
    return writeAuditRecord(user)
}

Go GOOD

func ProcessUser(user User) error {
    if err := validateUser(user); err != nil {
        return err
    }
    return writeAuditRecord(user)
}

func validateUser(user User) error {
    panic("example body omitted")
}

func writeAuditRecord(user User) error {
    panic("example body omitted")
}


Rust BAD

fn validate_user(user: &User) -> Result<(), ValidationError> {
    unimplemented!("example body omitted")
}

fn write_audit_record(user: &User) -> Result<(), AuditError> {
    unimplemented!("example body omitted")
}

fn process_user(user: &User) -> Result<(), ProcessError> {
    validate_user(user)?;
    write_audit_record(user)?;
    Ok(())
}

Rust GOOD

fn process_user(user: &User) -> Result<(), ProcessError> {
    validate_user(user)?;
    write_audit_record(user)?;
    Ok(())
}

fn validate_user(user: &User) -> Result<(), ValidationError> {
    unimplemented!("example body omitted")
}

fn write_audit_record(user: &User) -> Result<(), AuditError> {
    unimplemented!("example body omitted")
}


TypeScript BAD

function validateUser(user: User): void {
  throw new Error("example body omitted")
}

function writeAuditRecord(user: User): void {
  throw new Error("example body omitted")
}

function processUser(user: User): void {
  validateUser(user)
  writeAuditRecord(user)
}

TypeScript GOOD

function processUser(user: User): void {
  validateUser(user)
  writeAuditRecord(user)
}

function validateUser(user: User): void {
  throw new Error("example body omitted")
}

function writeAuditRecord(user: User): void {
  throw new Error("example body omitted")
}


Pattern

Put main logic first.

Put helper functions below the main operation.

Show the reader the shape before the details.


=====================================================================
3. AVOID UNNECESSARY ABSTRACTION
=====================================================================

Python BAD

class UserProcessor:
    def run(self, user):
        return user.save()

UserProcessor().run(user)

Python GOOD

user.save()


Go BAD

type Saver interface {
    Save() error
}

func Handle(saver Saver) error {
    return saver.Save()
}

Go GOOD

func Handle(user *User) error {
    return user.Save()
}


Rust BAD

trait Save {
    fn save(&self) -> Result<(), SaveError>;
}

fn process<T: Save>(item: T) -> Result<(), SaveError> {
    item.save()
}

Rust GOOD

fn process(user: &User) -> Result<(), SaveError> {
    user.save()
}


TypeScript BAD

function execute<T>(operation: () => T): T {
  return operation()
}

execute(() => saveUser(user))

TypeScript GOOD

saveUser(user)


Pattern

Do not add factories, managers, builders, wrappers, traits, or adapters unless they remove real complexity.


=====================================================================
4. KEEP EACH FUNCTION FOCUSED
=====================================================================

Python BAD

def handle_request(request):
    user = authenticate(request)
    data = load_user_data(user)
    send_email(user, data)
    return {"status": "ok"}

Python GOOD

def handle_request(request):
    user = authenticate(request)
    data = load_user_data(user)
    notify_user(user, data)
    return {"status": "ok"}


Go BAD

func Handle(request Request) (Response, error) {
    user, err := authenticate(request)
    if err != nil {
        return Response{}, err
    }

    data, err := loadUserData(user)
    if err != nil {
        return Response{}, err
    }

    if err := sendEmail(user, data); err != nil {
        return Response{}, err
    }

    return ResponseOK(), nil
}

Go GOOD

func Handle(request Request) (Response, error) {
    user, err := authenticate(request)
    if err != nil {
        return Response{}, err
    }

    data, err := loadUserData(user)
    if err != nil {
        return Response{}, err
    }

    if err := notifyUser(user, data); err != nil {
        return Response{}, err
    }

    return ResponseOK(), nil
}


Rust BAD

fn handle_request(request: Request) -> Result<Response, Error> {
    let user = authenticate(&request)?;
    let data = load_user_data(&user)?;
    send_email(&user, &data)?;
    Ok(Response::ok())
}

Rust GOOD

fn handle_request(request: Request) -> Result<Response, Error> {
    let user = authenticate(&request)?;
    let data = load_user_data(&user)?;
    notify_user(&user, &data)?;
    Ok(Response::ok())
}


TypeScript BAD

function handleRequest(request: Request): Response {
  const user = authenticate(request)
  const data = loadUserData(user)
  sendEmail(user, data)
  return Response.ok()
}

TypeScript GOOD

function handleRequest(request: Request): Response {
  const user = authenticate(request)
  const data = loadUserData(user)
  notifyUser(user, data)
  return Response.ok()
}


Pattern

Each function should have one clear responsibility.

The function name should describe that responsibility.


=====================================================================
5. PREFER EXPLICIT CODE
=====================================================================

Python BAD

valid = any(map(lambda item: item.is_valid(), items))

Python GOOD

def has_valid_item(items):
    for item in items:
        if item.is_valid():
            return True
    return False


Go BAD

return slices.ContainsFunc(items, func(item Item) bool {
    return item.Valid()
})

Go GOOD

for _, item := range items {
    if item.Valid() {
        return true
    }
}
return false


Rust BAD

items.iter().map(|item| item.is_valid()).any(|valid| valid)

Rust GOOD

fn has_valid_item(items: &[Item]) -> bool {
    for item in items {
        if item.is_valid() {
            return true;
        }
    }

    false
}


TypeScript BAD

const hasValidItem = items.some(item => item.isValid())

TypeScript GOOD

function hasValidItem(items: Item[]): boolean {
  for (const item of items) {
    if (item.isValid()) return true
  }

  return false
}


Pattern

Readable code is better than compact code that hides control flow.


=====================================================================
6. KEEP DEFENSIVE CODE SIMPLE
=====================================================================

Python BAD

def get_value(data):
    try:
        return data.get("value", None)
    except Exception:
        return None

Python GOOD

def get_value(data):
    if not isinstance(data, dict):
        return None
    return data.get("value")


Go BAD

func GetValue(data map[string]string) string {
    value, ok := data["value"]
    if ok {
        return value
    }
    return ""
}

Go GOOD

func GetValue(data map[string]string) (string, bool) {
    value, ok := data["value"]
    return value, ok
}


Rust BAD

fn get_value(data: Option<&BTreeMap<String, String>>) -> Option<String> {
    match data {
        Some(data) => match data.get("value") {
            Some(value) => Some(value.clone()),
            None => None,
        },
        None => None,
    }
}

Rust GOOD

fn get_value(data: Option<&BTreeMap<String, String>>) -> Option<String> {
    let data = data?;
    data.get("value").cloned()
}


TypeScript BAD

function getValue(data: unknown): string | undefined {
  try {
    return data["value"]
  } catch {
    return undefined
  }
}

TypeScript GOOD

function getValue(data: Record<string, string> | undefined): string | undefined {
  if (!data) return undefined
  return data.value
}


Pattern

Check the expected shape directly.

Do not hide unrelated errors.

Do not change data structures inside an example unless the data structure is the rule being taught.


=====================================================================
7. HANDLE ERRORS PREDICTABLY
=====================================================================

Python BAD

try:
    return load_config()
except:
    return {}

Python GOOD

try:
    return load_config()
except FileNotFoundError as error:
    raise ConfigMissing("config.json missing") from error


Go BAD

data, _ := os.ReadFile("config.json")

Go GOOD

data, err := os.ReadFile("config.json")
if err != nil {
    return nil, fmt.Errorf("read config.json: %w", err)
}


Rust BAD

fn load_config() -> Config {
    match std::fs::read_to_string("config.json") {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

Rust GOOD

fn load_config() -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string("config.json")
        .map_err(|source| ConfigError::ReadFailed { source })?;

    serde_json::from_str(&contents)
        .map_err(|source| ConfigError::ParseFailed { source })
}


TypeScript BAD

const config = JSON.parse(raw || "{}")

TypeScript GOOD

function parseConfig(raw: string): Config {
  try {
    return JSON.parse(raw)
  } catch (error) {
    throw new ConfigError("invalid config JSON", { cause: error })
  }
}


Pattern

Do not swallow failures.

Return or raise specific errors.

Preserve the original cause when the language supports it.


=====================================================================
8. AVOID HIDDEN STATE
=====================================================================

Python BAD

cache = {}

def compute(value):
    cache[value] = expensive(value)
    return cache[value]

Python GOOD

def compute(value, cache):
    result = expensive(value)
    cache[value] = result
    return result


Go BAD

var cache = map[int]int{}

func Compute(value int) int {
    cache[value] = Expensive(value)
    return cache[value]
}

Go GOOD

func Compute(value int, cache map[int]int) int {
    result := Expensive(value)
    cache[value] = result
    return result
}


Rust BAD

static CACHE: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn compute(value: &str) -> String {
    let result = expensive(value);
    CACHE.lock().unwrap().push(result.clone());
    result
}

Rust GOOD

fn compute(value: &str, cache: &mut Vec<String>) -> String {
    let result = expensive(value);
    cache.push(result.clone());
    result
}


TypeScript BAD

const cache: Record<string, string> = {}

function compute(value: string): string {
  cache[value] = expensive(value)
  return cache[value]
}

TypeScript GOOD

function compute(value: string, cache: Record<string, string>): string {
  const result = expensive(value)
  cache[value] = result
  return result
}


Pattern

Pass state explicitly.

Avoid hidden mutation.

Hidden state makes behavior harder to test.


=====================================================================
9. KEEP FILES AND FUNCTIONS SMALL
=====================================================================

BAD

user.rs

validation
permissions
notifications
audit
storage
api
serialization

GOOD

user/
  mod.rs
  validation.rs
  permissions.rs
  notifications.rs
  audit.rs
  repository.rs

Pattern

Split code by responsibility.

Large files hide complexity.

A file should have one clear reason to exist.


=====================================================================
10. SEPARATE ESSENTIAL COMPLEXITY FROM ACCIDENTAL COMPLEXITY
=====================================================================

Python BAD

class Strategy:
    def execute(self, value):
        raise NotImplementedError

class AddOne(Strategy):
    def execute(self, value):
        return value + 1

def run(strategy, value):
    return strategy.execute(value)

run(AddOne(), 5)

Python GOOD

def add_one(value):
    return value + 1


Go BAD

type Strategy interface {
    Execute(value int) int
}

type AddOne struct{}

func (AddOne) Execute(value int) int {
    return value + 1
}

func Run(strategy Strategy, value int) int {
    return strategy.Execute(value)
}

Go GOOD

func AddOne(value int) int {
    return value + 1
}


Rust BAD

trait Strategy {
    fn execute(&self, value: i32) -> i32;
}

struct AddOne;

impl Strategy for AddOne {
    fn execute(&self, value: i32) -> i32 {
        value + 1
    }
}

fn run(strategy: &dyn Strategy, value: i32) -> i32 {
    strategy.execute(value)
}

Rust GOOD

fn add_one(value: i32) -> i32 {
    value + 1
}


TypeScript BAD

interface Strategy {
  execute(value: number): number
}

class AddOne implements Strategy {
  execute(value: number): number {
    return value + 1
  }
}

function run(strategy: Strategy, value: number): number {
  return strategy.execute(value)
}

TypeScript GOOD

function addOne(value: number): number {
  return value + 1
}


Pattern

Simple problems should have simple solutions.

Add structure only when the problem requires it.


=====================================================================
11. DO NOT BLOCK THE ASYNC EXECUTOR
=====================================================================

Rust BAD

async fn fetch_and_save(url: &str) -> Result<(), Error> {
    let data = download_data(url).await?;
    std::fs::write("output.dat", data)?;
    Ok(())
}

Rust GOOD

async fn fetch_and_save(url: &str) -> Result<(), Error> {
    let data = download_data(url).await?;

    tokio::task::spawn_blocking(move || {
        std::fs::write("output.dat", data)
    })
    .await??;

    Ok(())
}

Pattern

Blocking operations must not stall async runtime threads.

Use async file I/O or move blocking work to a blocking thread pool.

Heavy CPU work also belongs outside the executor path.


=====================================================================
12. DO NOT HOLD SYNC LOCKS ACROSS AWAIT POINTS
=====================================================================

Rust BAD

use std::sync::Mutex;

async fn update_metrics(metrics: &Mutex<Metrics>) -> Result<(), Error> {
    let mut guard = metrics.lock().map_err(|_| Error::LockPoisoned)?;
    let value = fetch_metric_value().await?;
    guard.increment(value);
    Ok(())
}

Rust GOOD

use std::sync::Mutex;

async fn update_metrics(metrics: &Mutex<Metrics>) -> Result<(), Error> {
    let value = fetch_metric_value().await?;

    let mut guard = metrics.lock().map_err(|_| Error::LockPoisoned)?;
    guard.increment(value);

    Ok(())
}

Pattern

Keep lock scope short.

Never hold a synchronous lock guard across .await.

If state must be held across yield points, use an async-aware lock and verify the design is necessary.


=====================================================================
13. PREFER MANAGED CONCURRENCY OVER RAW SPAWNING
=====================================================================

Rust BAD

async fn process_batch(items: Vec<Item>) {
    for item in items {
        tokio::spawn(async move {
            persist_item(item).await;
        });
    }
}

Rust GOOD

use futures::stream::{self, StreamExt};

async fn process_batch(items: Vec<Item>) {
    stream::iter(items)
        .for_each_concurrent(10, |item| async move {
            persist_item(item).await;
        })
        .await;
}

Pattern

Concurrency should have explicit limits.

Avoid fire-and-forget task creation.

Unbounded task spawning can exhaust memory, overwhelm connection pools, and lose error visibility.


=====================================================================
14. DO NOT USE ASYNC FOR INSTANT OPERATIONS
=====================================================================

Rust BAD

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

async fn calculate_id(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

async fn handle_user(name: &str) {
    let id = calculate_id(name).await;
    save_id(id).await;
}

Rust GOOD

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_id(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

async fn handle_user(name: &str) {
    let id = calculate_id(name);
    save_id(id).await;
}

Pattern

Use async only when suspension is possible.

Cheap synchronous computation should stay synchronous.


=====================================================================
15. COMMENTS ARE AN EXCEPTION
=====================================================================

BAD

// Increment counter.
counter += 1;

// Return success.
return Ok(());

GOOD

counter += 1;

return Ok(());

Pattern

Comments are not a primary documentation mechanism.

Prefer better names, smaller functions, stronger types, and clearer control flow.

If a comment explains what code does, rewrite the code.


=====================================================================
16. DO NOT USE COMMENTS TO COMPENSATE FOR BAD NAMES
=====================================================================

BAD

// Validated users.
let x = Vec::new();

// Synchronization state.
let flag = false;

GOOD

let validated_users = Vec::new();

let synchronization_complete = false;

Pattern

Fix the names.

Do not explain the names.


=====================================================================
17. NEVER USE TODO COMMENTS
=====================================================================

BAD

// TODO: improve validation

// TODO: handle edge cases

// TODO: remove later

GOOD

Track future work in the issue tracker, roadmap, or backlog.

Pattern

Future work does not belong in source comments.

Do not leave TODO, FIXME, HACK, NOTE, or temporary reminder comments in source code.


=====================================================================
18. NEVER COMMENT OUT CODE
=====================================================================

BAD

// let result = old_algorithm(input);
// return result;

return new_algorithm(input);

GOOD

return new_algorithm(input);

Pattern

Version control preserves history.

Dead code must be deleted.


=====================================================================
19. DO NOT USE DECORATIVE COMMENTS
=====================================================================

BAD

// =====================
// Validation Section
// =====================

GOOD

fn validate_candidate(candidate: &Candidate) -> Result<(), ValidationError> {
    candidate.validate()
}

Pattern

Organization belongs in code structure.

Use modules, files, functions, and names.

Do not use comment banners.


=====================================================================
20. COMMENTS MAY EXPLAIN EXTERNAL CONSTRAINTS
=====================================================================

ACCEPTABLE

// Required for compatibility with protocol version 2.
message.flags |= LEGACY_FLAG;

ACCEPTABLE

// Replay protection requires monotonically increasing values.
sequence_number += 1;

ACCEPTABLE

// Vendor API returns HTTP 200 even when authorization fails.
if response.status == "error" {
    return Err(AuthError::Rejected);
}

Pattern

Comments may explain external constraints that code cannot express by itself.

Allowed comment subjects:

Security requirements
Protocol requirements
Vendor defects
Regulatory requirements
Non-obvious mathematical constraints

Comments must not narrate implementation details.


=====================================================================
21. WHEN IN DOUBT, DELETE THE COMMENT
=====================================================================

Rule

If removing a comment changes nothing, delete it.

If the code becomes unclear without the comment, rewrite the code first.

Keep the comment only when the reason cannot be expressed clearly in code.

Pattern

The default state of source code is no comment.


=====================================================================
END OF FILE
=====================================================================