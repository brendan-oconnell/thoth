use std::str::FromStr;
use thoth_api::model::language::Language;
use thoth_api::model::language::LanguageCode;
use thoth_api::model::language::LanguageRelation;
use thoth_errors::ThothError;
use uuid::Uuid;
use yew::html;
use yew::prelude::*;
use yew_agent::Dispatched;
use yewtil::fetch::Fetch;
use yewtil::fetch::FetchAction;
use yewtil::fetch::FetchState;
use yewtil::NeqAssign;

use crate::agent::notification_bus::NotificationBus;
use crate::agent::notification_bus::NotificationDispatcher;
use crate::agent::notification_bus::NotificationStatus;
use crate::agent::notification_bus::Request;
use crate::component::utils::FormBooleanSelect;
use crate::component::utils::FormLanguageCodeSelect;
use crate::component::utils::FormLanguageRelationSelect;
use crate::models::language::create_language_mutation::CreateLanguageRequest;
use crate::models::language::create_language_mutation::CreateLanguageRequestBody;
use crate::models::language::create_language_mutation::PushActionCreateLanguage;
use crate::models::language::create_language_mutation::PushCreateLanguage;
use crate::models::language::create_language_mutation::Variables;
use crate::models::language::delete_language_mutation::DeleteLanguageRequest;
use crate::models::language::delete_language_mutation::DeleteLanguageRequestBody;
use crate::models::language::delete_language_mutation::PushActionDeleteLanguage;
use crate::models::language::delete_language_mutation::PushDeleteLanguage;
use crate::models::language::delete_language_mutation::Variables as DeleteVariables;
use crate::models::language::language_codes_query::FetchActionLanguageCodes;
use crate::models::language::language_codes_query::FetchLanguageCodes;
use crate::models::language::language_relations_query::FetchActionLanguageRelations;
use crate::models::language::language_relations_query::FetchLanguageRelations;
use crate::models::language::LanguageCodeValues;
use crate::models::language::LanguageRelationValues;
use crate::string::CANCEL_BUTTON;
use crate::string::EMPTY_LANGUAGES;
use crate::string::NO;
use crate::string::REMOVE_BUTTON;
use crate::string::YES;

use super::ToElementValue;

pub struct LanguagesFormComponent {
    data: LanguagesFormData,
    new_language: Language,
    show_add_form: bool,
    fetch_language_codes: FetchLanguageCodes,
    fetch_language_relations: FetchLanguageRelations,
    push_language: PushCreateLanguage,
    delete_language: PushDeleteLanguage,
    notification_bus: NotificationDispatcher,
}

#[derive(Default)]
struct LanguagesFormData {
    language_codes: Vec<LanguageCodeValues>,
    language_relations: Vec<LanguageRelationValues>,
}

pub enum Msg {
    ToggleAddFormDisplay(bool),
    SetLanguageCodesFetchState(FetchActionLanguageCodes),
    GetLanguageCodes,
    SetLanguageRelationsFetchState(FetchActionLanguageRelations),
    GetLanguageRelations,
    SetLanguagePushState(PushActionCreateLanguage),
    CreateLanguage,
    SetLanguageDeleteState(PushActionDeleteLanguage),
    DeleteLanguage(Uuid),
    ChangeLanguageCode(LanguageCode),
    ChangeLanguageRelation(LanguageRelation),
    ChangeMainLanguage(bool),
}

#[derive(Clone, Properties, PartialEq)]
pub struct Props {
    pub languages: Option<Vec<Language>>,
    pub work_id: Uuid,
    pub update_languages: Callback<Option<Vec<Language>>>,
}

impl Component for LanguagesFormComponent {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let data: LanguagesFormData = Default::default();
        let show_add_form = false;
        let new_language: Language = Default::default();
        let fetch_language_codes = Default::default();
        let fetch_language_relations = Default::default();
        let push_language = Default::default();
        let delete_language = Default::default();
        let notification_bus = NotificationBus::dispatcher();

        ctx.link().send_message(Msg::GetLanguageCodes);
        ctx.link().send_message(Msg::GetLanguageRelations);

        LanguagesFormComponent {
            data,
            new_language,
            show_add_form,
            fetch_language_codes,
            fetch_language_relations,
            push_language,
            delete_language,
            notification_bus,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleAddFormDisplay(value) => {
                self.show_add_form = value;
                true
            }
            Msg::SetLanguageCodesFetchState(fetch_state) => {
                self.fetch_language_codes.apply(fetch_state);
                self.data.language_codes = match self.fetch_language_codes.as_ref().state() {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.language_codes.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetLanguageCodes => {
                ctx.link().send_future(
                    self.fetch_language_codes
                        .fetch(Msg::SetLanguageCodesFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetLanguageCodesFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetLanguageRelationsFetchState(fetch_state) => {
                self.fetch_language_relations.apply(fetch_state);
                self.data.language_relations = match self.fetch_language_relations.as_ref().state()
                {
                    FetchState::NotFetching(_) => vec![],
                    FetchState::Fetching(_) => vec![],
                    FetchState::Fetched(body) => body.data.language_relations.enum_values.clone(),
                    FetchState::Failed(_, _err) => vec![],
                };
                true
            }
            Msg::GetLanguageRelations => {
                ctx.link().send_future(
                    self.fetch_language_relations
                        .fetch(Msg::SetLanguageRelationsFetchState),
                );
                ctx.link()
                    .send_message(Msg::SetLanguageRelationsFetchState(FetchAction::Fetching));
                false
            }
            Msg::SetLanguagePushState(fetch_state) => {
                self.push_language.apply(fetch_state);
                match self.push_language.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.create_language {
                        Some(l) => {
                            let language = l.clone();
                            let mut languages: Vec<Language> =
                                ctx.props().languages.clone().unwrap_or_default();
                            languages.push(language);
                            ctx.props().update_languages.emit(Some(languages));
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            true
                        }
                        None => {
                            ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        ctx.link().send_message(Msg::ToggleAddFormDisplay(false));
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::CreateLanguage => {
                let body = CreateLanguageRequestBody {
                    variables: Variables {
                        work_id: ctx.props().work_id,
                        language_relation: self.new_language.language_relation,
                        language_code: self.new_language.language_code,
                        main_language: self.new_language.main_language,
                    },
                    ..Default::default()
                };
                let request = CreateLanguageRequest { body };
                self.push_language = Fetch::new(request);
                ctx.link()
                    .send_future(self.push_language.fetch(Msg::SetLanguagePushState));
                ctx.link()
                    .send_message(Msg::SetLanguagePushState(FetchAction::Fetching));
                false
            }
            Msg::SetLanguageDeleteState(fetch_state) => {
                self.delete_language.apply(fetch_state);
                match self.delete_language.as_ref().state() {
                    FetchState::NotFetching(_) => false,
                    FetchState::Fetching(_) => false,
                    FetchState::Fetched(body) => match &body.data.delete_language {
                        Some(language) => {
                            let to_keep: Vec<Language> = ctx
                                .props()
                                .languages
                                .clone()
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|l| l.language_id != language.language_id)
                                .collect();
                            ctx.props().update_languages.emit(Some(to_keep));
                            true
                        }
                        None => {
                            self.notification_bus.send(Request::NotificationBusMsg((
                                "Failed to save".to_string(),
                                NotificationStatus::Danger,
                            )));
                            false
                        }
                    },
                    FetchState::Failed(_, err) => {
                        self.notification_bus.send(Request::NotificationBusMsg((
                            ThothError::from(err).to_string(),
                            NotificationStatus::Danger,
                        )));
                        false
                    }
                }
            }
            Msg::DeleteLanguage(language_id) => {
                let body = DeleteLanguageRequestBody {
                    variables: DeleteVariables { language_id },
                    ..Default::default()
                };
                let request = DeleteLanguageRequest { body };
                self.delete_language = Fetch::new(request);
                ctx.link()
                    .send_future(self.delete_language.fetch(Msg::SetLanguageDeleteState));
                ctx.link()
                    .send_message(Msg::SetLanguageDeleteState(FetchAction::Fetching));
                false
            }
            Msg::ChangeLanguageRelation(val) => self.new_language.language_relation.neq_assign(val),
            Msg::ChangeLanguageCode(code) => self.new_language.language_code.neq_assign(code),
            Msg::ChangeMainLanguage(val) => self.new_language.main_language.neq_assign(val),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let languages = ctx.props().languages.clone().unwrap_or_default();
        let open_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(true)
        });
        let close_modal = ctx.link().callback(|e: MouseEvent| {
            e.prevent_default();
            Msg::ToggleAddFormDisplay(false)
        });
        html! {
            <nav class="panel">
                <p class="panel-heading">
                    { "Languages" }
                </p>
                <div class="panel-block">
                    <button
                        class="button is-link is-outlined is-success is-fullwidth"
                        onclick={ open_modal }
                    >
                        { "Add Language" }
                    </button>
                </div>
                <div class={ self.add_form_status() }>
                    <div class="modal-background" onclick={ &close_modal }></div>
                    <div class="modal-card">
                        <header class="modal-card-head">
                            <p class="modal-card-title">{ "New Language" }</p>
                            <button
                                class="delete"
                                aria-label="close"
                                onclick={ &close_modal }
                            ></button>
                        </header>
                        <section class="modal-card-body">
                            <form id="languages-form" onsubmit={ ctx.link().callback(|e: FocusEvent| {
                                e.prevent_default();
                                Msg::CreateLanguage
                            }) }
                            >
                                <FormLanguageCodeSelect
                                    label = "Language Code"
                                    value={ self.new_language.language_code }
                                    data={ self.data.language_codes.clone() }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeLanguageCode(LanguageCode::from_str(&e.to_value()).unwrap())
                                    ) }
                                    required = true
                                />
                                <FormLanguageRelationSelect
                                    label = "Language Relation"
                                    value={ self.new_language.language_relation }
                                    data={ self.data.language_relations.clone() }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeLanguageRelation(LanguageRelation::from_str(&e.to_value()).unwrap())
                                    ) }
                                    required = true
                                />
                                <FormBooleanSelect
                                    label = "Main"
                                    value={ self.new_language.main_language }
                                    onchange={ ctx.link().callback(|e: Event|
                                        Msg::ChangeMainLanguage(e.to_value() == "true")
                                    ) }
                                    required = true
                                />
                            </form>
                        </section>
                        <footer class="modal-card-foot">
                            <button
                                class="button is-success"
                                type="submit"
                                form="languages-form"
                            >
                                { "Add Language" }
                            </button>
                            <button
                                class="button"
                                onclick={ &close_modal }
                            >
                                { CANCEL_BUTTON }
                            </button>
                        </footer>
                    </div>
                </div>
                {
                    if !languages.is_empty() {
                        html!{{for languages.iter().map(|p| self.render_language(ctx, p))}}
                    } else {
                        html! {
                            <div class="notification is-warning is-light">
                                { EMPTY_LANGUAGES }
                            </div>
                        }
                    }
                }
            </nav>
        }
    }
}

impl LanguagesFormComponent {
    fn add_form_status(&self) -> String {
        match self.show_add_form {
            true => "modal is-active".to_string(),
            false => "modal".to_string(),
        }
    }

    fn render_language(&self, ctx: &Context<Self>, l: &Language) -> Html {
        let language_id = l.language_id;
        html! {
            <div class="panel-block field is-horizontal">
                <span class="panel-icon">
                    <i class="fas fa-language" aria-hidden="true"></i>
                </span>
                <div class="field-body">
                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Language Code" }</label>
                        <div class="control is-expanded">
                            {&l.language_code}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Language Relation" }</label>
                        <div class="control is-expanded">
                            {&l.language_relation}
                        </div>
                    </div>

                    <div class="field" style="width: 8em;">
                        <label class="label">{ "Main" }</label>
                        <div class="control is-expanded">
                            {
                                match &l.main_language {
                                    true => { YES },
                                    false => { NO }
                                }
                            }
                        </div>
                    </div>

                    <div class="field">
                        <label class="label"></label>
                        <div class="control is-expanded">
                            <a
                                class="button is-danger"
                                onclick={ ctx.link().callback(move |_| Msg::DeleteLanguage(language_id)) }
                            >
                                { REMOVE_BUTTON }
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}
