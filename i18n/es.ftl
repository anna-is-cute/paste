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

new = Nuevo
new-paste = Nuevo pegado
new-paste-desc = Crear un nuevo pegado.

## Paste attributes

paste-name = Nombre del pegado
  .placeholder = Nombre del pegado
paste-description = Descripción
  .placeholder = Descripción del pegado
paste-visibility = Visibilidad
  .public = público
  .public-desc = {paste-visibility.public} - visible para los buscadores y se muestra en su perfil público
  .unlisted = fuera de la lista
  .unlisted-desc = {paste-visibility.unlisted} - oculto para los buscadores y sólo los que tienen el enlace pueden verlo
  .private = privado
  .private-desc = {paste-visibility.private} - sólo es visible para usted cuando está conectado
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
  .description = Como no estás usando JavaScript, paste requiere que respondas a esta sencilla pregunta matemática para frustrar los robots simples. paste tiene un API si esta tratando de hacer cosas legítimas!
  .placeholder = Respuesta

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
  .placeholder = Código generado o código de respaldo

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
  .description = Manejar la configuración de la cuenta.

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

## API keys

api-keys = Claves de API
  .description = Manejar claves de API.

api-keys-table-header =
  .name = Nombre
  .key = Clave

api-keys-no-keys = ¡No tiene claves!

api-key-add =
  .name-placeholder = Nombre de clave
  .submit = Añadir

## Delete account

delete-account = Eliminar cueva
  .description = Elimine su cuenta aquí.

delete-account-warning = ¡Hala!
  .line-1 = Por favor, tómese su tiempo para leer este texto.
  .line-2 = La eliminación de su cuenta es <em>instantánea y permanente</em>. Todas sus pegados dejarán de ser accesibles inmediatamente y eventualmente serán eliminadas del servidor por una tarea.
  .line-3 = Si realmente desea eliminar su cuenta, escriba su contraseña en el cuadro de abajo y haga clic en el botón. Se le redirigirá a la página de inicio después de que se elimine su cuenta.

delete-account-password = Su contraseña segura
  .help = Escriba su contraseña para confirmar la eliminación de la cuenta.

delete-account-submit = Eliminar mi cuenta

## 2FA

tfa = 2FA
  .description = Manejar la autenticación de dos factores.

tfa-backup = Códigos de respaldo
  .body-1 = Si alguna vez pierde el acceso a su dispositivo 2FA, puede utilizar estos códigos en lugar de un código generado para acceder a su cuenta. Cada código sólo se puede utilizar una vez. Sólo se mostrarán una vez, así que no los pierda!
  .body-2 = Puede generar nuevos códigos en cualquier momento mientras esté conectado. Utilice el botón de abajo para hacerlo.

tfa-new-backup = Generar nuevos códigos de respaldo

tfa-enabled = 2FA está activado.
tfa-disabled = 2FA no está activado.

tfa-turn-off = Desactivar 2FA
tfa-turn-on = Activar 2FA

tfa-explanation =
  .part-1 = La autenticación de dos factores (una forma de <a class="external" target="_blank" href="https://es.wikipedia.org/wiki/Autenticaci%C3%B3n_de_m%C3%BAltiples_factores">autenticación de múltiples factores</a>) aumenta la seguridad de su cuenta al requerirle que especifique una contraseña única generada por otro dispositivo al iniciar sesión.
  .part-2 = Esta característica requiere un dispositivo inteligente o tarjeta inteligente y una aplicación para generar los códigos. Para los dispositivos Android e iOS, consulte la tabla siguiente.

tfa-devices =
  .play-store = Android (Play Store)
  .ios = iOS (App Store)
  .fdroid = Android (F-Droid)

## Enable 2FA

tfa-enable = Activar 2FA
  .description = Activar la autenticación de dos factores.

tfa-enable-scan-qr = Escanee el código QR que aparece a continuación utilizando su aplicación de autenticación. Si no puede escanear el código, puede introducir el secreto compartido manualmente, <strong>sin espacios</strong>:

tfa-enable-new-secret = Si desea generar un nuevo secreto compartido, haga clic en el botón de abajo.
  .button = Regenerar el secreto

tfa-enable-enter-code = Después de configurar la aplicación de autenticación, introduzca el código que ha generado a continuación.
  .placeholder = Código 2FA
  .button = Activar

## Disable 2FA

tfa-disable = Desactivar 2FA
  .description = Desactivar la autenticación de dos factores.

tfa-disable-warning = Desactivar 2FA reducirá la seguridad de su cuenta. Si aún así desea desactivarlo, introduzca su contraseña a continuación.

tfa-disable-password =
  .placeholder = Su contraseña segura
  .button = Desactivar

## Credits

credits = Créditos
  .description = Si paste fuera una película, estas serían al final.

## Errors

error-400 = Mala solicitud.
  .description = Hizo una solicitud incorrecta.

error-403 = Prohibido.
  .description = No se le permite ver o hacer lo que sea que estaba tratando de ver o hacer.

error-404 = Página no encontrada.
  .description = No pudimos encontrar lo que buscaba. Lo siento!

error-500 = Error interno del servidor.
  .description = También conocido como "metimos la pata". Esto siempre es un error, así que siéntese libre de <a href="https://github.com/jkcclemens/paste/issues/new?title=500%20on%20%3Croute%3E&body=So%20I%20was%20trying%20to%20go%20to%20%3Croute%3E,%20but%20then%20the%20server%20was%20all%20like%20nah,%20bb,%20500!">reportarlo</a>!

error-other = Error.
  .description = Tiene un error extraño.

error-csrf = Token anti-CSRF no válido.

## Admin

admin-success = Ahora usted es un administrador.

## Admin errors

admin-no-key = No se ha configurado ninguna clave de administrador.
admin-bad-key = Clave incorrecta.
admin-already-admin = Usted ya es un administrador.
admin-exists = No puede convertirte en un administrador de esta manera si los administradores ya existen.

## Admin nav

admin-tabs =
  .overview = Panorama general
  .pastes = Pegados
  .users = Usuarios
  .config = Config
  .maintenance = Mantenimiento

## Admin overview

admin-overview =
  .title = Admin
  .subtitle = Realizar tareas de administración.

admin-stats =
  .pastes = Pegados
  .users = Usuarios

## Admin pastes

admin-pastes =
  .title = Pegados
  .subtitle = Manejar todos los pegados.

admin-pastes-list-headers =
  .link = Enlace
  .name = Nombre
  .description = Descripción
  .visibility = Visibilidad
  .author = Autor
  .files = Archivos
  .created = Creado
  .expires = Expira
  .actions = Acciones

admin-pastes-list =
  .anonymous = anónimo
  .empty = vacíos
  .never = nunca

admin-paste-delete =
  .success = Pegado eliminado.
  .missing = No pude encontrar esa pasta. ¿Ya se ha eliminado?

admin-paste-delete-modal =
  .title = Eliminar pegado como admin
  .confirm = Por favor, confirme que desea eliminar esta pegado como administrador.
  .button = Eliminar como admin

admin-batch-delete = Eliminar lote
  .desc = Insertar IDs o URLs de pegado, uno por línea, para eliminar.
  .placeholder = IDs o URLs de pegado, uno por línea
  .button = Eliminar
  .error = No se pudo eliminar el pegado <code>{$id}</code>: {$error}.
  .success = Eliminado {$pastes} {$pastes ->
    [one] pegado
   *[other] pegados
  }.

admin-batch-delete-missing = no pudo encontrar el pegado

## Admin users

admin-users =
  .title = Usuarios
  .subtitle = Manejar todos los usuarios.

admin-users-table =
  .name = Nombre
  .username = Nombre de usuario
  .email = Correo electrónico
  .email-verified = Correo electrónico verificado
  .tfa-enabled = 2FA activado
  .admin = Admin
  .actions = Acciones

admin-users-yes-no =
  .yes = Sí
  .no = No

admin-users-admin =
  .super = super
  .normal = admin
  .none = ninguno

admin-users-delete =
  .missing = No existe tal usuario.
  .success = Usuario eliminado.
  .super = No se puede eliminar un superadministrador.
  .other-admin = No se puede eliminar otro administrador.

admin-users-delete-modal =
  .title = Eliminar usuario
  .confirm = Por favor, confirme que desea eliminar a este usuario.
  .button = Eliminar

admin-users-promote-modal =
  .title = Ascender a <em>{$user}</em>
  .line-1 = ¿A qué nivel de administrador quiere ascender a <em>{$user}</em>?
  .superadmin = Superadmin
  .superadmin-desc = Los superadmins no pueden ser eliminados y pueden cambiar el estado del
                     administrador de otros usuarios. Son capaces de controlarlo todo.
  .superadmin-warning = <strong class="has-text-danger">Importante</strong>: La única manera de
                        degradar un superadmin es acceder a la base de datos directamente.
  .admin = Admin
  .admin-desc = Los administradores pueden ser eliminados y no pueden controlar el estado del
                administrador de otros usuarios. Los administradores no pueden eliminar a otros
                administradores.
  .button = Ascender

admin-users-demote-modal =
  .title = Degradar a <em>{$user}</em>
  .desc = Los usuarios degradados se convierten en usuarios normales sin privilegios de
          administrador.
  .button = Degradar

admin-users-status =
  .must-be-super = Debe ser un superadmin para ascender/degradar usuarios.
  .missing = No existe tal usuario.
  .target-super = No se puede ascender/degradar un superadmin.
  .already-admin = Ese usuario ya es un administrador.
  .not-admin = Ese usuario no es un administrador.
  .promoted = El usuario ha sido ascendido con éxito.
  .demoted = El usuario ha sido degradado con éxito.
  .invalid-level = Nivel de administrador inválido.

## Admin config

admin-config =
  .title = Config
  .subtitle = Editar la configuración del sitio.

admin-config-save = Guardar
