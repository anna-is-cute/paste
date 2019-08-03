## Nav

nav-new = New
nav-about = About
nav-log-in = Log in
nav-log-out = Log out
nav-register = Register
nav-settings = Settings
nav-swap-theme = Swap theme

## Footer

footer-code = code
# $version (string) version of the server
footer-server-version = server: {$version}
# $version (string) version of the resources
footer-resources-version = resources: {$version}
footer-credits = credits

## Avatar

# $username (string) username associated with the avatar
avatar-alt = {$username} avatar

## Home

new-paste = New paste
new-paste-desc = Create a new paste.

## Paste attributes

paste-name = Paste name
  .placeholder = Paste name
paste-description = Description
  .placeholder = Paste description
paste-visibility = Visibility
  .public = public
  .unlisted = unlisted
  .private = private
paste-expires = Expires
  .no = no
  .relative = relative
  .absolute = absolute

paste-absolute-expiry =
  .date = Expiry date
  .time = Expiry time
  .timezone = Expiry timezone

paste-relative-expiry =
  .years = Years
  .days = Days
  .hours = Hours
  .minutes = Minutes

## Antispam

# $a (number) first number to add
# $b (number) second number to add
antispam-math = What is {$a} + {$b}?
  .why = Why?
  .description = Since you're not using JavaScript, paste requires you to answer this simple math question to foil simple bots. paste has an API if you're trying to do legitimate things!

antispam-honeypot =
  .email = Email
  .title = Title

## Paste UI

paste-add-file = Add file
paste-submit-anonymous = Submit anonymously
paste-submit = Submit

## Paste display

disp-untitled = untitled paste
# $files (number) number of files the paste has
disp-num-files = {$files} {$files ->
  [one] file
 *[other] files
}
# put directly before the paste expiration date
# rendered, it ends up like "expires in 10 minutes"
disp-expires = expires

disp-tab-files = files
disp-tab-revisions = revisions

disp-delete-title = Delete paste
disp-delete-description = To delete an anonymous paste, you need to use its deletion key. Enter it below and click "{disp-delete-button}" to delete this paste.
disp-delete-key-placeholder = Deletion key
disp-delete-button = Delete
disp-delete-confirm = Please confirm you would like to delete this paste.

disp-dkey-title = Deletion key
disp-dkey-msg-1 = In order to delete this paste, you will need to use the key below. This message will only appear once.
disp-dkey-msg-2 = Note that this key is saved on <em>this browser only</em> for 30 days. If you clear the cache, the key will be lost.

disp-file-tab-rendered = Rendered
disp-file-tab-source = Source
disp-file-raw = Raw
disp-file-binary-content = Binary content

## Revisions

revisions-page-title =
  .named = Revisions for ${name}
  .unnamed = Revisions for {disp-untitled}

revisions-title =
  .named = Revisions for <span class="keeps-spaces">${name}</span>
  .unnamed = Revisions for <em>{disp-untitled}</em>

revisions-subtitle = View the changes made to this paste.

revisions-unknown-file = unknown file

## Edit

edit-page-title =
  .named = Edit {$name}
  .unnamed = Edit {disp-untitled}

edit-title =
  .named = Edit <span class="keeps-spaces">{$name}</span>
  .unnamed = Edit <em>{disp-untitled}</em>

edit-subtitle = Change this paste<span class="requires-js"> and its files</span>.

## User page

# $name (string) user's name
user-title = {$name}'s pastes
# $pastes (number) number of pastes the user has
user-num-pastes = {$pastes} {$pastes ->
  [one] paste
 *[other] pastes
}
# $name (string) user's name
user-no-pastes = It looks like {$name} has no pastes!

user-delete-selected = Delete selected
user-delete = Delete pastes
  .body = Please confirm you would like to delete the selected pastes.
  .button = Delete all

user-select = Select
  .all = All
  .none = None

## Pagination

pagination =
  .next = Next
  .previous = Previous

## File attributes

file-name-placeholder = File name with extension
file-language-auto = auto
file-hello-world = Hello world!

## General inputs

input-what-is-this = What is this?

## About

about-title = About
# $siteName (string) name of this paste instance
about-blurb =
  .before = {$siteName} runs on
  .paste = paste,
  .after = an open-source project dedicated to making a sensible, modern pastebin that anyone can host.

## Log in

login = Log in
  .description = Log in to your account.

login-username = Username
  .placeholder = jimbo123

login-password = Password
  .placeholder = Your secure password

login-2fa = 2FA code
  .placeholder = If enabled

login-forgot-password = Forgot your password?

login-submit = Log in

## Register

register = Register
  .description = Create a new account.

register-display-name = Display name
  .placeholder = Jim Bob Jones

register-username = Username
  .placeholder = jimbo123

register-password = Password
  .placeholder = Something really secure, please <3

register-password-again = Password (again)
  .placeholder = What you typed above

register-email = Email
  .placeholder = your@email.com

register-submit = Submit

## Forgot password

forgot = Forgot password
  .description = Start the password reset process.

forgot-email = Email
  .placeholder = your@email.com

forgot-submit = Submit

## Reset password

reset = Reset password
  .description = Reset your password here.

reset-password = Password
  .placeholder = Something really secure, please <3

reset-password-again = Password (again)
  .placeholder = What you typed above

reset-submit = Submit

## Settings

settings-tabs =
  .settings = Settings
  .api-keys = API Keys
  .two-factor = 2FA
  .delete = Delete

## Account

account = Account
  .description = Manage account settings.

email-not-verified = Your email is not verified!
  .body = Click the button below to resend a verification email.
  .button = Send verification email

account-display-name = Display name

account-username = Username
  .help = Changing your username will invalidate all links with your username in them.

account-avatar = Avatar
  .help = Your avatar is determined using an avatar provider linked to your email address. Changes can be made to your avatar at whichever provider you have chosen.

account-avatar-provider = Avatar provider
  .link = link

account-email = Email

account-change-password = Change password
  .help = Leave this blank to keep your password the same.

account-change-password-again = Change password (again)

account-current-password = Current password
  .help = Required to change any information.

account-submit = Change

## API keys

api-keys = API keys
  .description = Manage API keys.

api-keys-table-header =
  .name = Name
  .key = Key

api-keys-no-keys = You have no keys!

api-key-add =
  .name-placeholder = Key name
  .submit = Add

## Delete account

delete-account = Delete account
  .description = Delete your account here.

delete-account-warning = Whoa there!
  .line-1 = Please take the time to read this text!
  .line-2 = Deleting your account is <em>instantaneous and permanent</em>. All of your pastes will immediately cease being accessible and will eventually be deleted off of the server by a task.
  .line-3 = If you really want to delete your account, type your password in the box below and click the button. You will be redirected to the home page after your account is deleted.

delete-account-password = Your secure password
  .help = Type your password to confirm account deletion.

delete-account-submit = Delete my account
