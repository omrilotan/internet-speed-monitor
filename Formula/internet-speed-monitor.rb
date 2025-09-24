cask "internet-speed-monitor" do
  version "1.1.2"
  sha256 :no_check

  url "https://github.com/omrilotan/internet-speed-monitor/releases/download/v#{version}/Internet.Speed.Monitor-#{version}.dmg",
      verified: "github.com/omrilotan/internet-speed-monitor"
  name "Internet Speed Monitor"
  desc "Monitor internet connectivity speed at set intervals"
  homepage "https://github.com/omrilotan/internet-speed-monitor"

  livecheck do
    url :url
    strategy :github_latest
  end

  auto_updates true

  app "Internet Speed Monitor.app"

  zap trash: [
    "~/Library/Application Support/internet-speed-monitor",
    "~/Library/Preferences/com.omrilotan.internet-speed-monitor.plist",
    "~/Library/Saved Application State/com.omrilotan.internet-speed-monitor.savedState",
  ]
end