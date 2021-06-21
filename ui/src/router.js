import Vue from 'vue'
import Router from 'vue-router'
import DownloadConfig from './views/DownloadConfig.vue'
import MigrateView from './views/MigrateView.vue'
import SelectPackages from './views/SelectPackages.vue'
import ErrorView from './views/ErrorView.vue'
import InstallPackages from './views/InstallPackages.vue'
import CompleteView from './views/CompleteView.vue'
import ModifyView from './views/ModifyView.vue'
import AuthenticationView from './views/AuthenticationView.vue'
import ReAuthenticationView from './views/ReAuthenticationView.vue'

Vue.use(Router)

export default new Router({
  routes: [
    {
      path: '/config',
      name: 'config',
      component: DownloadConfig
    },
    {
      path: '/migrate',
      name: 'migrate',
      component: MigrateView
    },
    {
      path: '/packages',
      name: 'packages',
      component: SelectPackages
    },
    {
      path: '/install/:kind',
      name: 'install',
      component: InstallPackages
    },
    {
      path: '/showerr',
      name: 'showerr',
      component: ErrorView
    },
    {
      path: '/complete/:uninstall/:update/:migrate/:packages_installed',
      name: 'complete',
      component: CompleteView
    },
    {
      path: '/modify',
      name: 'modify',
      component: ModifyView
    },
    {
      path: '/authentication',
      name: 'authentication',
      component: AuthenticationView
    },
    {
      path: '/reauthenticate',
      name: 'reauthenticate',
      component: ReAuthenticationView
    },
    {
      path: '/',
      redirect: '/config'
    }
  ]
})
