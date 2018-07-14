require 'sidekiq'
require 'ffi'

class DeleteDirectory
  extend FFI::Library
  ffi_lib 'libworker_delete_directory'
  attach_function :delete_directory, [ :string ], :void

  include Sidekiq::Worker

  def perform(path)
    DeleteDirectory.delete_directory(path)
  end
end

class Email
  extend FFI::Library
  ffi_lib 'libworker_email'
  attach_function :email, [ :string, :string, :string, :string ], :void

  include Sidekiq::Worker

  def perform(config_path, email, subject, content)
    Email.email(config_path, email, subject, content)
  end
end

class Queue
  include Sidekiq::Worker

  def perform(clazz, timestamp, args)
    obj = Object::const_get(clazz)
    obj.perform_at(timestamp, *args)
  end
end

class ExpirePaste
  extend FFI::Library
  ffi_lib 'libworker_expire_paste'
  attach_function :expire_paste, [ :int64, :string, :string, :string ], :void

  include Sidekiq::Worker

  def perform(timestamp, store_path, user_id, paste_id)
    ExpirePaste.expire_paste(timestamp, store_path, user_id, paste_id)
  end
end
