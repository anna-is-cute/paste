require 'sidekiq'
require 'fileutils'

class DeleteAllPastes
  include Sidekiq::Worker

  def perform(path)
    if !Dir.exists?(path)
      return
    end

    FileUtils.remove_entry_secure(path)
  end
end
