use maud::{html, Markup, PreEscaped, DOCTYPE};

pub fn page(content: Markup, scripts: Option<Markup>) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" class="no-js" hx-ext="morph" {

        head {
          meta charset="UTF-8";
          meta name="viewport" content="width=device-width, initial-scale=1";
          meta name="google" content="notranslate";

          title {"Tasks App"}

          script type="module" {
            (PreEscaped("
                document.documentElement.classList.remove('no-js');
                document.documentElement.classList.add('js');
              "))
          }
          link href="/tailwind.css" rel="stylesheet";
          link href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.10.5/font/bootstrap-icons.css" rel="stylesheet";

          meta name="description" content="Page description";
          link rel="canonical" href="https://bootspoon.com";

        }

        // We need 100% height to make the mobile sidebar work
        //  Although Chrome defaults the background to white, Firefox defaults it to gray
        body class="min-h-full bg-white px-2 py-1" {
          header
            class="py-2 mb-2 -mx-2 px-2 border-b-2 border-gray-300"
            role="banner"
            data-cy="top-bar"
          {
          /*
            nowrap fixes grid doing min-content and making multi-word
            buttons be only one word wide.

            With items-center, the turtle button will have zero height and
            will thus be invisible (like Imhotep). So use items-stretch to
            force it to the parent height.
            */
            ul class="flex gap-x-2 items-center whitespace-nowrap" {
              header class="font-semibold text-lg mr-auto flex" {
                img
                  class="w-6 h-6 mr-2"
                  src="/favicon/android-chrome-512x512.png"
                  aria-hidden;
                a href="/" {"Space Traders"}
              }

            /*
              @match user {
                None => (anonymous_menu()),
                Some(_) => (user_menu()),
              }
              */
            }
          }

          main class="mx-4 my-2" hx-target-error="main" {(content)}

          script
            //src="https://unpkg.com/htmx.org@1.9.6"
            src="https://unpkg.com/htmx.org@1.9.6/dist/htmx.js"
            crossorigin="anonymous"
          {}
          script src="https://unpkg.com/idiomorph/dist/idiomorph-ext.min.js"{}

          script src="https://unpkg.com/hyperscript.org@0.9.8" {}

          script src="//unpkg.com/alpinejs" defer {}

            script src="https://cdn.jsdelivr.net/npm/unpoly@3.5.2/unpoly.min.js" {}
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/unpoly@3.5.2/unpoly.min.css";
            script {"up.poly.enable()"}

            script{(PreEscaped(r#"
            document.body.addEventListener('htmx:responseError', function (evt) {
                document.querySelector("main").innerText =
                    "Something went wrong. It's probably our fault. Please reload the page and try again.";
            });
            "#))}

          script src="/scripts.js" {}

          @if let Some(scripts) = scripts {
            (scripts)
          }

          script {(PreEscaped(r"
            // Copied from the web-push documentation
            const urlBase64ToUint8Array = (base64String) => {
              const padding = '='.repeat((4 - base64String.length % 4) % 4);
              const base64 = (base64String + padding)
                .replace(/\-/g, '+')
                .replace(/_/g, '/');

              const rawData = window.atob(base64);
              const outputArray = new Uint8Array(rawData.length);

              for (let i = 0; i < rawData.length; ++i) {
                outputArray[i] = rawData.charCodeAt(i);
              }
              return outputArray;
            };

            (async () => {
              if ('serviceWorker' in navigator) {
                navigator.serviceWorker.register('/sw.js', {
                  scope: '/',
                });
                const registration = await navigator.serviceWorker.ready;
                const subscription = await registration.pushManager.subscribe({
                  userVisibleOnly: true,
                  applicationServerKey: urlBase64ToUint8Array('BLx-q_uJpcwePUvIXEiJbqUPXpHWd41ow-oAkHcUmjq5JWTbHNylWCi5KNoDhts-WJRhNpMnIuqmZn0_vjwGIb8')
                });
                await fetch(
                  '/web_push/subscribe',
                  {
                    method: 'POST',
                    body: JSON.stringify(subscription),
                    headers: {'content-type': 'application/json',
                  },
                });
              }
            })();
          "))}
        }
      }
    }
}
