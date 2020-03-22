[![CircleCI](https://circleci.com/gh/spamwax/alfred-pinboard-rs.svg?style=svg)](https://circleci.com/gh/spamwax/alfred-pinboard-rs)
 [![Build status](https://ci.appveyor.com/api/projects/status/3g5uvuocs52yc444/branch/master?svg=true)](https://ci.appveyor.com/project/spamwax/alfred-pinboard-rs/branch/master) [![Build Status](https://travis-ci.org/spamwax/alfred-pinboard-rs.svg?branch=master)](https://travis-ci.org/spamwax/alfred-pinboard-rs)


# Alfred Workflow for Pinboard


Manage, post and **preview** your bookmarks on [Pinboard](https://pinboard.in) right from within [Alfred app](https://www.alfredapp.com).
### Important: [Update & Support for Alfred 3 Users](#alfred_3_support)
### Alfred 4: If upgrading workflow from `0.14.14` or before, you may have to directly [download](https://github.com/spamwax/alfred-pinboard-rs/releases/latest) latest version and then open it in Finder

## Features
Pinboard is a great and reliable bookmarking service. Its [front page](https://pinboard.in) sums it all:
"**Social Bookmarking for Introverts. Pinboard is a fast, no-nonsense bookmarking site.**"

This plugin will let you:

- _**post**_ a bookmark to Pinboard right from Alfred, with:
  - Fetching bookmark information from active browser's window
  - _tag_ auto-completion to show your current tags.
  - _popular_ tags for the current _url_
  - and more ...
- _**search**_ your current bookmarks
  - Tap <kbd>Shift</kbd> to show a preview of selected item without opening browser.
  - Tap <kbd>Command+L</kbd> to show _Large_ toast of title
  - Tap <kbd>Command</kbd> to show current item's _tags_
  - Tap <kbd>Control</kbd> to show current item's extended notes/descriptioin.
- Automatic [updates](#misc) of workflow
- Many options that can be easily adjusted. (see below)

### TLDR;
After initial [setup](#installation--setup):
- For posting you just need to enter the Workflow's keyword ( `p` ) into Alfred's window and follow it with couple of tags and an optional description. The workflow will then post a bookmark for the window/tab of the active browser to Pinboard.

- For searching, use ( `ps` ) and then type the search keywords.

### Supported Browsers:
- Safari
- Chromium
- Firefox (See [known issues](#known_issues) )
- Opera
- Vivaldi
- Brave
- qutebrowser (See [known issues](#known_issues) )
- Firefox Dev. Edition
- Safari Tech. Preview
- Chrome

## Installation / Setup
After [downloading](https://github.com/spamwax/alfred-pinboard-rs/releases/latest) the latest version of the workflow and installing it in Alfred, you need to do a one-time setup to authenticate the Workflow. This Workflow only uses username/token method so you won't need to enter your password. (This is the *suggested* way of using Pinboard's API).
If you don't have a token, get one from Pinbaord's [setting's page](https://pinboard.in/settings/password).

Then invoke Alfred and enter your username:token after the ***"pa"*** keyword:

![image](./res/images/authentication.png)

This workflow will keep a local cache of the tags and bookmarks that you have in Pinboard, and, by default, automatically update the cache. (See [manual updates](#cache_update) )

---

## Usage (post a bookmark):
The syntax to post a bookmark to Pinboard is :

```
p tag1 tag2 tag3 ; some optional note (semi-colon & note are optional)
```

The workflow will show a list of your current tags as you are typing:

![image](./res/images/non-fuzzy-search-tags.png)

The number below each tag shows how many times you have used it in Pinboard bookmarks.
You can move Alfred's highlighter to the desired tag and hit '**Tab**' to **autocomplete** it.

To finish the process just press Enter.

- If tag suggestion feature is enabled (see `pset suggest_tags`), 3 popular tags based on current active webpage will be added to the list of your tags. The list is fetched from Pinboard's API and is often helpful. However this feature will add a 1 second delay to showing the tag list after first keystroke. This delay is disabled for consequent keystrokes as the fetched popular tags are cached.

  ![image](./res/images/popular-tags.png)

#### Modifiers (<kbd>Control ⌃, Option ⌥</kbd>)
You can hold down modifiers to one-time change some of your settings:

- <kbd>Control ⌃</kbd> : will toggle `toread` setting momentarily.
- <kbd>Option ⌥</kbd> : will toggle `shared` setting momentarily.


After entering last desired tag, you can hit enter to post the bookmark.
If you want to add extra description to the bookmark you can add it after a semi-colon:

![image](./res/images/adding-notes.png)

#### Already saved bookmark
If the current page is already saved, you will be notified. However the workflow assumes that:

- `http://example.com/list.html`
- `https://example.com/list.html`
- `http://example.com/list.html#fragment`

are all different bookmarks and will not notify you about duplication!

  ![image](./res/images/already-saved.png)

---

## Usage (search bookmarks):
Searching your bookmarks is easy.

```
ps query1 query2 query3 ...
```

Workflow will use the text you enter in Alfred and show list of bookmarks that contain **all** of the search keywords in any of the bookmarks information (Description of bookmark, its tags, its url and its extended notes). However, these search fields can be adjusted, see [settings](#config).

So **the more** search keywords you enter **the less** results will be displayed as it tries to find the bookmarks that contain ***all*** of the keywords.

The search result is ordered in descending order of dates they were posted to your Pinboard account.

![image](./res/images/bookmarks-search-results.png)

To show tags (instead of URLS) in search results subtitles, use `pset url_tag`:

![image](./res/images/url_vs_tag.png)

#### Modifier keys (<kbd>Command ⌘, Control ⌃, Option ⌥</kbd>)
You can hold down modifiers to enable different behavior:

- <kbd>Control ⌃</kbd> : will show the extended description of selected bookmark.
- <kbd>Command ⌘</kbd> : will show tags of selected bookmark.
- <kbd>Option ⌥</kbd> : Holding `⌥` and pressing enter will open the bookmark in [Pinboard's website](https://pinboard.in).
- <kbd>Command ⌘</kbd> + <kbd>Option ⌥</kbd> : Hitting Return ⏎ key with these combo modifiers will copy the bookmark's URL to clipboard.
- <kbd>Shift ⇧</kbd>: **Tap** ⇧ to load a preview of bookmark without opening your browser 😎 ⤵︎

![image](./res/images/quicklook-preview.png)

---

## Usage (delete a bookmark):
```
pind
```

To delete a bookmark, just make sure it is opened in your current browser's window. Then use `pind`.
Proposed workflow for deleting a bookmark is to first open it in browser (search for it using this workflow's keyword `ps`, and hit enter) and then use `pind` keyword followed by enter.

![image](./res/images/delete-pin.png)

## Usage (rename a tag):
```
pr old_tag new_tag
```

**Note:** Pinboard tag renaming API lacks providing error message if `old_tag` is not in your tag collection!!! It basically says `success` even though no renaming was done!


## Settings<a name="config"></a>

You can configure the behavior of workflow by entering `pconf` in Alfred:

![image](./res/images/configuration.png)

Selecting each setting and hitting ⏎ (<kbd>Enter</kbd>) will let you adjust it:

![image](./res/images/set-fuzzy.png)

On top of using `pconf`, you can directly type following commands to also adjust the settings:

- `pset fuzzy`: Enable/disable fuzzy search.
- `pset suggest_tags`: When posting a new bookmark, list popular tags for the active page. Note that this information is fetched from Pinboard and sometimes is not very _accurate_.
- `pset shared`: Mark all new bookmarks as _shared_.
- `pset toread`: Mark all new bookmarks as _toread_.
- `pset check_bookmarked`: Notify if active page is already bookmarked.
- `pset tagonly`: Only search within _tag_ field while doing any look-up.
- `pset auto`: After posting a new bookmark, automatically update the local cache.
- `pset tags`: Set number of tags to show: `pset tags 25`
- `pset bookmarks`: Set number of bookmarks to show: `pset bookmarks 12`
- `pset url_tag`: Show either URLs or tags in search results subtitles

### Manual cache update<a name="cache_update"></a>

To manually update the cache, you need to issue the ***`pu`*** command:

![image](./res/images/update.png)

---

Most of configuration settings are self-explanatory. However `fuzzy` search may need a demo.

When fuzzy search is enabled, the tags/bookmarks that contain the query letters in the given order are displayed:

  ![image](./res/images/fuzzy-search-tags.png)

  Otherwise, _normal_ search will search for consecutive characters in query:

  ![image](./res/images/non-fuzzy-search-tags.png)

---

## Misc.<a name="misc"></a>
- Workflow will check for newever version of itself every 24 hours. This check only takes place when you actually use one of its keywords (no background service is ever used). You can also manually check for updates using `pcheck`.

![image](./res/images/upgrade_available.png)

- This workflow tries to show some helpful errors in different cases.

![image](./res/images/error-1.png)
![image](./res/images/error-2.png)
![image](./res/images/error-3.png)

- If you want to change some behavior take a look at Alfred's workflow page:

![image](./res/images/workflow-screenshot.png)

---

## Known Issues<a name="known_issues"></a>
- **Firefox**: While tag suggestions and "Check if page is bookmarked" options are enabled, posting bookmark from Firefox is broken. Deleting bookmarks using `pind` won't work when Firefox is your active browser. Reason: Firefox does not properly support interacting with it programatically.
- This workflow is setup to work with Aflred 4. It may or may not work with previous version as it has not been tested for them.

## Alfred 3 Support & Issues<a name="alfred_3_support"></a>
- Alfred 4 has made some changes to internal structure of workflows. Therefore if you updated this workflow through the built-in *automatic update*, it's very *likely* that it'll stop working. If that happens, you can fix it by first deleteing & then re-installing the workflow by [downloading the last version (0.14.11)](https://github.com/spamwax/alfred-pinboard-rs/releases/tag/0.14.11) that has Alfred 3 format.
- Moving forward, new features, changes & bug fixes will first target Alfred 4 users and then (time permitting) will be ported back to Alfred 3.

## TODO

I wish to add the following in the coming releases:

- ~~Let users delete a selected bookmark from witin Alfred.~~
- ~~Add a proper logging facility to Rust code.~~ (uses log_env)
- ~~Use a better error mechanism (maybe [failure](https://crates.io/crates/failure)?)~~

## Feedback / Bugs
This is my first non-trivial project using Rust language so so your [feedback or bug](https://github.com/spamwax/alfred-pinboard-rs/compare) reports are greatly appreciated.

## License
This open source software is licensed under [MIT License](./LICENSE.md).
