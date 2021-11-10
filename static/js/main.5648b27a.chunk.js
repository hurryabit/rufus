(this["webpackJsonprufus-ui"]=this["webpackJsonprufus-ui"]||[]).push([[0],{14:function(e,a,t){},16:function(e,a,t){},17:function(e,a,t){"use strict";t.r(a);var n=t(0),l=t.n(n),r=t(5),s=t.n(r),c=(t(14),t(1)),o=t.n(c),i=t(6),m=t(3),u=t(4),d=t(8),h=t(7),f=(t(16),function(e){Object(d.a)(n,e);var a=Object(h.a)(n);function n(e){var l;return Object(m.a)(this,n),(l=a.call(this,e)).loadWasm=Object(i.a)(o.a.mark((function e(){var a;return o.a.wrap((function(e){for(;;)switch(e.prev=e.next){case 0:return e.prev=0,e.next=3,t.e(3).then(t.bind(null,19));case 3:a=e.sent,l.setState({wasm:a}),e.next=10;break;case 7:e.prev=7,e.t0=e.catch(0),console.error("Unexpected error in loadWasm. [Message: ".concat(e.t0.message,"]"));case 10:case"end":return e.stop()}}),e,null,[[0,7]])}))),l.handleProgramChange=function(e){l.setState({program:e.currentTarget.value})},l.handleProgramKeyDown=function(e){"Enter"===e.key&&e.metaKey&&l.handleRunClick(e)},l.handleRunClick=function(e){e.preventDefault();var a=l.state.wasm;if(a){var t=a.exec(l.state.program),n=t.status,r=t.get_value();switch(n){case a.ExecResultStatus.Ok:l.setState({result:r});break;case a.ExecResultStatus.Err:alert(r)}}else alert("WASM not loaded!")},l.state={wasm:null,program:"let twice = fun f x -> f (f x) in\nlet inc = fun x -> x + 1 in\ntwice inc 0",output:"",result:""},l}return Object(u.a)(n,[{key:"componentDidMount",value:function(){this.loadWasm()}},{key:"render",value:function(){var e=this.state;return l.a.createElement(l.a.Fragment,null,l.a.createElement("section",{className:"hero is-info"},l.a.createElement("div",{className:"hero-body"},l.a.createElement("div",{className:"container"},l.a.createElement("h1",{className:"title"},"rufus"),l.a.createElement("h2",{className:"subtitle"},"An experiment about a CEK machine implemented in Rust, compiled to Web Assembly and made alive via Typescript + React.")))),l.a.createElement("section",{className:"section"},l.a.createElement("div",{className:"container"},l.a.createElement("div",{className:"columns"},l.a.createElement("div",{className:"column is-two-thirds"},l.a.createElement("div",{className:"field"},l.a.createElement("label",{className:"label"},"Program"),l.a.createElement("div",{className:"control"},l.a.createElement("textarea",{className:"textarea is-family-code",spellCheck:!1,rows:15,value:e.program,onChange:this.handleProgramChange,onKeyDown:this.handleProgramKeyDown}))),l.a.createElement("div",{className:"field"},l.a.createElement("label",{className:"label"},"\xa0"),l.a.createElement("div",{className:"control"},l.a.createElement("button",{className:"button is-fullwidth is-info",onClick:this.handleRunClick},"Run")))),l.a.createElement("div",{className:"column is-one-third"},l.a.createElement("div",{className:"field"},l.a.createElement("label",{className:"label"},"Output"),l.a.createElement("div",{className:"control"},l.a.createElement("textarea",{className:"textarea is-family-code",readOnly:!0,rows:15,value:e.output}))),l.a.createElement("div",{className:"field"},l.a.createElement("label",{className:"label"},"Result"),l.a.createElement("div",{className:"control"},l.a.createElement("input",{className:"input is-family-code",type:"text",readOnly:!0,value:e.result}))))))),l.a.createElement("footer",{className:"footer"},l.a.createElement("div",{className:"content has-text-centered"},"\xa9 2019\u20132021 ",l.a.createElement("a",{href:"https://github.com/hurryabit/rufus",target:"blank"},"Martin Huschenbett"))))}}]),n}(l.a.Component));Boolean("localhost"===window.location.hostname||"[::1]"===window.location.hostname||window.location.hostname.match(/^127(?:\.(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}$/));s.a.render(l.a.createElement(f,null),document.getElementById("root")),"serviceWorker"in navigator&&navigator.serviceWorker.ready.then((function(e){e.unregister()}))},9:function(e,a,t){e.exports=t(17)}},[[9,1,2]]]);
//# sourceMappingURL=main.5648b27a.chunk.js.map