var port = browser.runtime.connectNative("alfred_pinboard_rs");

/*
On a tab change, send the tab URL to the app
*/
function updateTab() {
  function postMessage(tabs) {
    const tab = tabs[0]
    const url = tab && tab.url;
    if (url && url.match(/^https?:/)) {
      console.log("Sending: " + url);
      port.postMessage(url);
    } else {
      console.log("Skipping: " + url);
    }
  }
  var gettingActiveTab = browser.tabs.query({active: true, currentWindow: true});
  gettingActiveTab.then(postMessage);
}

// TODO: is this list necessary? sufficient?
browser.tabs.onActivated.addListener(updateTab);
browser.tabs.onUpdated.addListener(updateTab);
browser.windows.onFocusChanged.addListener(updateTab);

