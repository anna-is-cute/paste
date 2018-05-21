require 'sidekiq'
require 'ffi'

class DeleteAllPastesLib
  extend FFI::Library
  ffi_lib 'libworker_delete_all_pastes'
  attach_function :delete_all_pastes, [ :string ], :void
end

class DeleteAllPastes
  include Sidekiq::Worker

  def perform(path)
    DeleteAllPastesLib.delete_all_pastes(path)
  end
end

class EmailLib
  extend FFI::Library
  ffi_lib 'libworker_email'
  attach_function :email, [ :string, :string, :string, :string, :string ], :void
end

class Email
  include Sidekiq::Worker

  def perform(config_path, email, name, subject, content)
    EmailLib.email(config_path, email, name, subject, content)
  end
end
