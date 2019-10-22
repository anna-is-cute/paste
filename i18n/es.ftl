## Nav

nav =
  .new = Nuevo
  .about = Sobre
  .log-in = 	Iniciar sesión
  .log-out = Cerrar sesión
  .register = Registrarse
  .settings = Ajustes
  .admin = Admin
  .swap-theme = Cambiar el tema

## Footer

footer-code = código
# $version (string) version of the server
footer-server-version = servidor: {$version}
# $version (string) version of the resources
footer-resources-version = recursos: {$version}
footer-credits = créditos

## Avatar

# $username (string) username associated with the avatar
avatar-alt = {$username} avatar

## Home

new-paste = Nuevo pegado
new-paste-desc = Crear un nuevo pegado.

## Paste attributes

paste-name = Nombre del pegado
  .placeholder = Nombre del pegado
paste-description = Descripción
  .placeholder = Descripción del pegado
paste-visibility = Visibilidad
  .public = público
  .unlisted = fuera de la lista
  .private = privado
paste-expires = Expira
  .no = no
  .relative = relativo
  .absolute = absoluto

paste-absolute-expiry =
  .date = Fecha de vencimiento
  .time = Hora de vencimiento
  .timezone = Zona horaria de vencimiento

paste-relative-expiry =
  .years = Años
  .days = Días
  .hours = Horas
  .minutes = Minutos

## Antispam

# $a (number) first number to add
# $b (number) second number to add
antispam-math = ¿Qué es {$a} + {$b}?
  .why = Por qué?
  .description = Como no estás usando JavaScript, paste requiere que respondas a esta sencilla pregunta matemática para frustrar los robots simples. paste tiene un API si estás tratando de hacer cosas legítimas!

antispam-honeypot =
  .email = Correo electrónico
  .title = Título

## Paste UI

paste-add-file = Añadir archivo
paste-submit-anonymous = Enviar anónimamente
paste-submit = Enviar

## Paste display

disp-untitled = pegado sin título
# $files (number) number of files the paste has
disp-num-files = {$files} {$files ->
  [one] archivo
 *[other] archivos
}
# put directly before the paste expiration date
# rendered, it ends up like "expires in 10 minutes"
disp-expires = expira

disp-tab-files = archivos
disp-tab-revisions = revisiónes

disp-delete-title = Eliminar pegado
disp-delete-description = Para eliminar una pegado anónimo, debe utilizar su clave de eliminación. Introdúzcalo a continuación y haga clic en "{disp-delete-button}" para eliminar este pegado.
disp-delete-key-placeholder = Clave de eliminación
disp-delete-button = Eliminar
disp-delete-confirm = Por favor, confirme que desea eliminar este pegado.

disp-dkey-title = Clave de eliminación
disp-dkey-msg-1 = Para eliminar este pegado, deberá utilizar la siguiente clave. Este mensaje sólo aparecerá una vez.
disp-dkey-msg-2 = Tenga en cuenta que esta clave se guarda en <em>este navegador sólo</em> durante 30 días. Si borra la caché, la clave se perderá.

disp-file-tab-rendered = Renderizado
disp-file-tab-source = Fuente
disp-file-raw = Crudo
disp-file-binary-content = Contenido binario

## Revisions

revisions-page-title =
  .named = Revisiones para {$name}
  .unnamed = Revisiones para {disp-untitled}

revisions-title =
  .named = Revisiones para <span class="keeps-spaces">{$name}</span>
  .unnamed = Revisiones para <em>{disp-untitled}</em>

revisions-subtitle = Ver los cambios realizados en esta pegado.

revisions-unknown-file = archivo desconocido

## Edit

edit-page-title =
  .named = Editar {$name}
  .unnamed = Editar {disp-untitled}

edit-title =
  .named = Editar <span class="keeps-spaces">{$name}</span>
  .unnamed = Editar <em>{disp-untitled}</em>

edit-subtitle = Cambiar esta pegar<span class="requires-js"> y sus archivos</span>.

## User page

# $name (string) user's name
user-title = Pegados de {$name}
# $pastes (number) number of pastes the user has
user-num-pastes = {$pastes} {$pastes ->
  [one] pegado
 *[other] pegados
}
# $name (string) user's name
user-no-pastes = ¡Parece que {$name} no tiene pegados!

user-delete-selected = Eliminar seleccionados
user-delete = Eliminar pegados
  .body = Confirme que desea eliminar los pegados seleccionados.
  .button = Eliminar todos

user-select = Seleccionar
  .all = Todos
  .none = Ninguno

## Pagination

pagination =
  .next = Siguiente
  .previous = Anterior

## File attributes

file-name-placeholder = Nombre del archivo con extensión
file-language-auto = auto
file-hello-world = Hola mundo!

## About

about-title = Sobre
# $siteName (string) name of this paste instance
about-blurb =
  .before = {$siteName} trabaja con
  .paste = paste,
  .after = un proyecto de código abierto dedicado a hacer un pastebin sensible y moderno que cualquiera puede alojar.

## Log in

login = Iniciar sesión
  .description = Iniciar sesión en su cuenta.

login-username = Nombre de usuario
  .placeholder = jimbo123

login-password = Contraseña
  .placeholder = Su contraseña segura

login-2fa = Código 2FA
  .placeholder = Si está activado

login-forgot-password = ¿Olvidó su contraseña?

login-submit = Iniciar sesión

## Register

register = Inscribirse
  .description = Crear una nueva cuenta.

register-display-name = Nombre para mostrar
  .placeholder = Jim Bob Jones

register-username = Nombre de usuario
  .placeholder = jimbo123

register-password = Contraseña
  .placeholder = Algo realmente seguro, por favor <3

register-password-again = Contraseña (otra vez)
  .placeholder = Lo que escribió arriba

register-email = Correo electrónico
  .placeholder = su@correo.com

register-submit = Enviar

## Forgot password

forgot = Olvidó su contraseña
  .description = Iniciar el proceso de restablecimiento de contraseña.

forgot-email = Correo electrónico
  .placeholder = su@correo.com

forgot-submit = Enviar

## Reset password

reset = Restablecer su contraseña
  .description = Restablecer su contraseña aquí.

reset-password = Contraseña
  .placeholder = Algo realmente seguro, por favor <3

reset-password-again = Contraseña (otra vez)
  .placeholder = Lo que escribió arriba

reset-submit = Enviar

## Settings

settings-tabs =
  .settings = Ajustes
  .api-keys = Claves de API
  .two-factor = 2FA
  .delete = Eliminar

## Account

account = Cuenta
  .description = Administrar la configuración de la cuenta.

email-not-verified = Su correo electrónico no está verificado!
  .body = Haga clic en el botón de abajo para volver a enviar un correo electrónico de verificación.
  .button = Enviar correo electrónico de verificación

account-display-name = Nombre para mostrar

account-username = Nombre de usuario
  .help = Cambiar su nombre de usuario invalidará todos los enlaces con su nombre de usuario en ellos.

account-avatar = Avatar
  .help = Su avatar se determina utilizando un proveedor de avatares vinculado a su dirección de correo electrónico. Se pueden hacer cambios en su avatar en cualquier proveedor que haya elegido.

account-avatar-provider = Proveedor de avatares
  .link = enlace

account-email = Correo electrónico

account-change-password = Cambiar contraseña
  .help = Deje este espacio en blanco para mantener su contraseña igual.

account-change-password-again = Cambiar contraseña (otra vez)

account-current-password = Contraseña actual
  .help = Requerido para cambiar cualquier información.

account-submit = Cambiar
